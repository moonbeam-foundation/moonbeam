# Store/expose relay block hash and relay slot number in parachain-system

Moon{beam, river} is planning to use the `relay chain block hash + relay slot number` as the input for a VRF. AFAICT neither of these values is persisted in `parachain-system` so we need to store and expose them.

get using `relay_chain::well_known_keys::CURRENT_SLOT` once relay chain proof is included in cumulus (after 0.9.18)

instead of the relay block hash, I'm thinking of using the PersistedValidationData::hash()...
