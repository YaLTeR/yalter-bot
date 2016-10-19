#[derive(Serialize, Deserialize)]
struct Memory {
	// The map is from ServerId into an array of RoleIds.
	admin_roles: BTreeMap<String, Vec<u64>>
}
