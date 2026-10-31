use crate::components::*;
use crate::events::{ContactEvent, DamageEvent};
use bevy::prelude::*;

/// Naive O(n^2) circle-overlap detection between hostile teams. Placeholder
/// until the spatial-partition collision module lands in `src/collision`.
/// Emits `ContactEvent` only; damage is a separate step.
pub fn detect_contacts(
    query: Query<(Entity, &Transform2D, &Collision, &Team)>,
    mut contacts: MessageWriter<ContactEvent>,
) {
    let mut combos = query.iter_combinations();
    while let Some([(ea, ta, ca, team_a), (eb, tb, cb, team_b)]) = combos.fetch_next() {
        if !team_a.is_hostile_to(*team_b) {
            continue;
        }
        let reach = ca.radius + cb.radius;
        if ta.position.distance_squared(tb.position) <= reach * reach {
            contacts.write(ContactEvent { a: ea, b: eb });
        }
    }
}

/// Converts contacts into damage: each side's `Damage` hurts the other.
/// A collider whose `Damage` is zero (or otherwise not positive) is harmless
/// by design and emits no event.
pub fn contacts_to_damage(
    mut contacts: MessageReader<ContactEvent>,
    damagers: Query<&Damage>,
    mut damage_events: MessageWriter<DamageEvent>,
) {
    for contact in contacts.read() {
        for (source, target) in [(contact.a, contact.b), (contact.b, contact.a)] {
            let Ok(damage) = damagers.get(source) else {
                continue;
            };
            if damage.value > 0.0 {
                damage_events.write(DamageEvent {
                    target,
                    amount: damage.value,
                    kind: damage.kind,
                    source: Some(source),
                });
            }
        }
    }
}
