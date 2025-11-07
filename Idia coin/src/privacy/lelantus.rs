use curve25519_dalek::{Scalar, RistrettoPoint};
use merlin::Transcript;
use rand_core::{RngCore, OsRng};

pub struct LelantusParameters {
    pub generators: Vec<RistrettoPoint>,
    pub h: RistrettoPoint,
    pub epoch_length: u64,
}

pub struct SparkNote {
    pub value: u64,
    pub randomness: Scalar,
    pub commitment: RistrettoPoint,
    pub nullifier: Scalar,
}

pub struct SpendProof {
    pub nullifier: Scalar,
    pub proof: BulletproofRangeProof,
    pub signature: SchnorrSignature,
}

pub struct MintProof {
    pub commitment: RistrettoPoint,
    pub range_proof: BulletproofRangeProof,
}

pub struct LelantusProtocol {
    params: LelantusParameters,
    merkle_tree: SparseMerkleTree,
    note_commitments: Vec<RistrettoPoint>,
    nullifier_set: HashSet<Scalar>,
}

impl LelantusProtocol {
    pub fn mint(&mut self, value: u64) -> Result<(SparkNote, MintProof), PrivacyError> {
        let mut rng = OsRng;
        
        // Generate randomness
        let randomness = Scalar::random(&mut rng);
        
        // Create commitment
        let commitment = self.commit_value(value, randomness)?;
        
        // Generate range proof
        let (range_proof, _) = self.prove_range(value, randomness)?;
        
        // Create note
        let note = SparkNote {
            value,
            randomness,
            commitment,
            nullifier: self.derive_nullifier(&commitment),
        };
        
        // Create proof
        let proof = MintProof {
            commitment,
            range_proof,
        };
        
        // Update state
        self.note_commitments.push(commitment);
        self.merkle_tree.insert(commitment.compress().to_bytes());
        
        Ok((note, proof))
    }
    
    pub fn spend(
        &mut self,
        note: SparkNote,
        recipient: PublicKey,
    ) -> Result<SpendProof, PrivacyError> {
        // Verify note exists
        if !self.merkle_tree.contains(&note.commitment.compress().to_bytes()) {
            return Err(PrivacyError::NoteNotFound);
        }
        
        // Check nullifier not already spent
        if self.nullifier_set.contains(&note.nullifier) {
            return Err(PrivacyError::NullifierAlreadySpent);
        }
        
        // Generate range proof
        let (range_proof, _) = self.prove_range(note.value, note.randomness)?;
        
        // Generate signature
        let signature = self.sign_spend(&note, &recipient)?;
        
        // Create proof
        let proof = SpendProof {
            nullifier: note.nullifier,
            proof: range_proof,
            signature,
        };
        
        // Update nullifier set
        self.nullifier_set.insert(note.nullifier);
        
        Ok(proof)
    }
    
    fn commit_value(&self, value: u64, randomness: Scalar) -> Result<RistrettoPoint, PrivacyError> {
        let value_scalar = Scalar::from(value);
        let commitment = self.params.h * randomness + 
                        self.params.generators[0] * value_scalar;
        Ok(commitment)
    }
    
    fn prove_range(
        &self,
        value: u64,
        randomness: Scalar,
    ) -> Result<(BulletproofRangeProof, Vec<RistrettoPoint>), PrivacyError> {
        let mut transcript = Transcript::new(b"lelantus-range-proof");
        
        let (proof, commitments) = RangeProof::prove_multiple(
            &mut transcript,
            &mut OsRng,
            64,
            &[value],
            &self.params.generators,
            &randomness,
            self.params.h,
        )?;
        
        Ok((proof, commitments))
    }
    
    fn derive_nullifier(&self, commitment: &RistrettoPoint) -> Scalar {
        let mut hasher = Blake2b::new();
        hasher.update(&commitment.compress().to_bytes());
        Scalar::from_hash(hasher)
    }
    
    fn sign_spend(
        &self,
        note: &SparkNote,
        recipient: &PublicKey,
    ) -> Result<SchnorrSignature, PrivacyError> {
        let mut rng = OsRng;
        let keypair = KeyPair::generate(&mut rng);
        
        let message = {
            let mut hasher = Blake2b::new();
            hasher.update(&note.nullifier.to_bytes());
            hasher.update(&recipient.to_bytes());
            Scalar::from_hash(hasher)
        };
        
        Ok(keypair.sign(message))
    }
}