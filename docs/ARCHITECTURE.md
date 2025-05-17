# Backend Architecture – Overview

This Mermaid diagram shows the main structure of the backend, its modules, and their interactions.

---

```mermaid
flowchart TD
    subgraph Config
        CFG[config/]
    end

    subgraph Server
        SRV[server/]
        subgraph Matchmaking
            MMK[matchmaking/]
        end
        subgraph GameSession
            GMS[game_session/]
        end
        STA[state.rs]
        ROU[router.rs]
    end

    subgraph Game
        GM[game/]
        TYP[types.rs]
        STA2[state.rs]
        SYS[systems/]
        ENT[entities/]
        GRD[grid/]
        UTL[utils.rs]
        DEMO[demo/]
        TST[tests.rs]
    end

    UTS[utils/]
    MAIN[main.rs]
    TOML[Cargo.toml]

    %% Relations
    MAIN --> SRV
    MAIN --> CFG
    MAIN --> GM

    SRV --> MMK
    SRV --> GMS
    SRV --> STA
    SRV --> ROU

    MMK <--> GMS
    MMK --> CFG
    GMS --> GM

    GM --> TYP
    GM --> STA2
    GM --> SYS
    GM --> ENT
    GM --> GRD
    GM --> UTL
    GM --> DEMO
    GM --> TST

    SRV --> GM
    SRV --> UTS
    GM --> UTS

    TOML --> MAIN

    classDef folder fill:#e3e3ff,stroke:#333,stroke-width:1px;
    class CFG,SRV,GM,UTS folder;
```

---

**Legend:**

- Boxes represent main folders/files.
- Arrows indicate primary dependencies or calls.
- Subgraphs show the hierarchy of modules.

**Update this section if you add or move modules.**
