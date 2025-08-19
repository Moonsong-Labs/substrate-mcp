/// Scaffold pallet handlebars template
pub const PROMPT: &str = r#"Create a complete Substrate pallet scaffold based on the following description:

<PALLET DESCRIPTION>
{{pallet_description}}
</PALLET DESCRIPTION>

## Implementation Requirements

### Workspace Integration
If the pallet is part of a workspace make sure it is compatible with its
dependencies. If it uses some dependency that's already in the worspace,
use the workspace dependeny (setting `{workspace = true}`)

### Runtime Integration

If the repository is for a substrate chain/s, add the pallet to its runtimes
unless specified otherwise in the PALLET_DESCRIPTION.
If the runtime hash generated weights and a way to run benchmarks, 
adapt this pallet to that flow and give instructions on how get proper pallet
 weights and integrate them into the runtime.

### Pallet Structure 
Check existing pallets in the workspace and and do a best effort to 
follow that structure. 
To fill in missing blanks, also check kitchensink pallet: https://github.com/paritytech/polkadot-sdk/tree/master/substrate/frame/examples/kitchensink


## Implementation Guidelines

1. **Storage Design**
   - Use appropriate storage types (Value, Map, DoubleMap)
   - Consider storage costs and access patterns
   - Add proper getters with documentation

2. **Error Handling**
   - Define specific, descriptive errors
   - Use `ensure!` for validation
   - Return early on errors

3. **Events**
   - Emit events for all state changes
   - Include relevant data for indexing
   - Document event meanings

4. **Weights**
   - Benchmark all extrinsics
   - Use realistic worst-case scenarios
   - Update weights after changes

5. **Testing**
   - Test all success paths
   - Test all error conditions
   - Test edge cases and boundaries
   - Test event emissions
   - Make sure tests pass when run from the workspace

## References
- Basic pallet structure: https://docs.polkadot.com/develop/parachains/customize-parachain/make-custom-pallet/
- Testing guide: https://docs.polkadot.com/develop/parachains/testing/pallet-testing/
- Benchmarking: https://docs.polkadot.com/develop/parachains/testing/benchmarking/"#;