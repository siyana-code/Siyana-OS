# Page table levels (PML4/PDPT/PD/PT)

A 4-level tree: PML4 → PDPT → PD → PT → 4KB page. In our simple setup each PDPT entry is a 1GB huge page, identity-mapping the first 4GB.
