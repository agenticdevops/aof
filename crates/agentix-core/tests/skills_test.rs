//! Tests for the SkillRegistry — built-in skill pack discovery.
//! TDD: these tests define required behavior before implementation.

use agentix_core::skills::SkillRegistry;

#[test]
fn test_skill_registry_has_8_builtin_packs() {
    let registry = SkillRegistry::new();
    let packs = registry.builtin_packs();
    assert_eq!(
        packs.len(),
        8,
        "Should have exactly 8 built-in skill packs, got {}",
        packs.len()
    );
}

#[test]
fn test_skill_registry_has_all_required_names() {
    let registry = SkillRegistry::new();
    let packs = registry.builtin_packs();
    let names: Vec<&str> = packs.iter().map(|p| p.name).collect();

    let expected = ["aws", "kubernetes", "terraform", "docker", "git", "database", "security", "observability"];
    for expected_name in &expected {
        assert!(
            names.contains(expected_name),
            "Missing required skill pack: '{}'. Found: {:?}",
            expected_name,
            names
        );
    }
}

#[test]
fn test_skill_registry_packs_have_content() {
    let registry = SkillRegistry::new();
    for pack in registry.builtin_packs() {
        assert!(
            pack.content.len() > 200,
            "Skill pack '{}' content is too short ({} chars) — must have substantive content",
            pack.name,
            pack.content.len()
        );
    }
}

#[test]
fn test_skill_registry_packs_have_descriptions() {
    let registry = SkillRegistry::new();
    for pack in registry.builtin_packs() {
        assert!(
            !pack.description.is_empty(),
            "Skill pack '{}' has an empty description",
            pack.name
        );
    }
}

#[test]
fn test_get_builtin_finds_by_name() {
    let registry = SkillRegistry::new();
    let pack = registry.get_builtin("kubernetes");
    assert!(
        pack.is_some(),
        "get_builtin('kubernetes') should return Some"
    );
    let pack = pack.unwrap();
    assert_eq!(pack.name, "kubernetes");
    assert!(
        pack.content.contains("kubectl"),
        "Kubernetes skill should mention kubectl"
    );
}

#[test]
fn test_get_builtin_returns_none_for_unknown() {
    let registry = SkillRegistry::new();
    let result = registry.get_builtin("nonexistent-skill-xyz");
    assert!(
        result.is_none(),
        "get_builtin('nonexistent-skill-xyz') should return None"
    );
}

#[test]
fn test_skill_registry_default_equals_new() {
    let registry1 = SkillRegistry::new();
    let registry2 = SkillRegistry::default();
    // Both should have the same number of packs
    assert_eq!(
        registry1.builtin_packs().len(),
        registry2.builtin_packs().len(),
        "SkillRegistry::default() should have same packs as SkillRegistry::new()"
    );
}
