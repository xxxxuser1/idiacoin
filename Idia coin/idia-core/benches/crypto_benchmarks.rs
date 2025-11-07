use criterion::{criterion_group, criterion_main, Criterion};
use idia_core::crypto::{
    PedersenCommitment,
    RangeProofWrapper,
    StealthAddress,
    RingSignature,
    KeyImage,
};
use curve25519_dalek::scalar::Scalar;
use rand::rngs::OsRng;

fn bench_pedersen_commit(c: &mut Criterion) {
    c.bench_function("pedersen_commit", |b| {
        b.iter(|| {
            let value = 1000u64;
            let (commitment, _) = PedersenCommitment::new(value);
            criterion::black_box(commitment);
        });
    });
}

fn bench_range_proof(c: &mut Criterion) {
    c.bench_function("range_proof", |b| {
        b.iter(|| {
            let value = 1000u64;
            let (proof, commitment) = RangeProofWrapper::new(value).unwrap();
            criterion::black_box((proof, commitment));
        });
    });
}

fn bench_stealth_address(c: &mut Criterion) {
    let recipient = StealthAddress::new();
    let mut rng = OsRng;
    
    c.bench_function("stealth_address_create", |b| {
        b.iter(|| {
            let r = Scalar::random(&mut rng);
            let (R, P) = recipient.generate_one_time_key(r);
            criterion::black_box((R, P));
        });
    });

    c.bench_function("stealth_address_scan", |b| {
        let r = Scalar::random(&mut rng);
        let (R, P) = recipient.generate_one_time_key(r);
        b.iter(|| {
            criterion::black_box(recipient.scan_one_time_key(&R, &P));
        });
    });
}

fn bench_ring_signature(c: &mut Criterion) {
    let mut rng = OsRng;
    let mut public_keys = Vec::new();
    let mut secret_keys = Vec::new();
    
    // Generate test keys
    for _ in 0..11 {
        let secret = Scalar::random(&mut rng);
        let public = &curve25519_dalek::constants::RISTRETTO_BASEPOINT_POINT * &secret;
        secret_keys.push(secret);
        public_keys.push(public);
    }
    
    let real_idx = 5;
    let key_image = KeyImage((public_keys[real_idx]).compress());

    c.bench_function("ring_signature_sign", |b| {
        b.iter(|| {
            let sig = RingSignature::sign(
                secret_keys[real_idx],
                key_image.clone(),
                &public_keys,
                real_idx,
            ).unwrap();
            criterion::black_box(sig);
        });
    });

    let sig = RingSignature::sign(
        secret_keys[real_idx],
        key_image.clone(),
        &public_keys,
        real_idx,
    ).unwrap();

    c.bench_function("ring_signature_verify", |b| {
        b.iter(|| {
            criterion::black_box(sig.verify(&public_keys).unwrap());
        });
    });
}

criterion_group!(
    benches,
    bench_pedersen_commit,
    bench_range_proof,
    bench_stealth_address,
    bench_ring_signature
);
criterion_main!(benches);