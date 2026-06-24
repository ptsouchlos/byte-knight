use shadow_rs::ShadowBuilder;

fn main() {
    ShadowBuilder::builder()
        .build_pattern(shadow_rs::BuildPattern::RealTime)
        .build()
        .unwrap();
}
