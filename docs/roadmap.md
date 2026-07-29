# Merra Roadmap: A Living Pixel Kingdom Simulation in Rust and Bevy

## Working vision

Build an extraordinarily deep pixel-art simulation game in which kingdoms do not merely provide a backdrop for the player. They are living historical systems.

Dynasties rise and disappear. Families migrate. Villages become towns. Roads redirect commerce. Religious movements divide communities. Noble houses accumulate grudges across generations. Wars begin for understandable reasons, then produce consequences nobody intended. Legends form from real events but become distorted as they pass through memory, rumor, propaganda and folklore.

The player will inhabit this world at human scale while also participating in history.

They might begin as:

* a minor noble;
* a village reeve;
* an itinerant scholar;
* a merchant;
* a mercenary captain;
* an abbey steward;
* a royal official;
* the founder of a new settlement;
* or an ordinary person whose descendants eventually matter.

The world should continue without the player. History should not wait for a quest trigger.

The long-term aspiration is:

> Build a simulated fantasy civilization capable of producing centuries of coherent history, then allow the player to enter that history and alter it from within.

This should be ambitious enough to support years of development. However, every development cycle must still end with something observable, playable, explainable and publishable.

---

# 1. The project’s two products

This project produces two things in parallel.

## Product A: The game

A pixel simulation game built with Rust and the Bevy ecosystem.

## Product B: The development chronicle

A recurring public account of the project published through the **Rusting** newsletter.

Every development cycle should create:

* a working software increment;
* a tagged code snapshot;
* a reproducible simulation seed;
* screenshots, GIFs or chronicle excerpts;
* extensive technical and design notes;
* a clear question for the next cycle.

The newsletter is not merely marketing added after development. It is part of
the project’s operating system. Cycle notes accumulate the evidence; each Era
ends with a substantial synthesis for the Rusting newsletter.

It will:

* force each cycle to produce a comprehensible result;
* document architectural decisions;
* establish a public history of the game;
* attract Rust and simulation enthusiasts;
* create accountability without requiring arbitrary ship dates;
* turn bugs and strange emergent behavior into material;
* gradually build an audience years before release.

The game creates the newsletter’s stories. The newsletter gives shape and rhythm to the game’s development.

---

# 2. Why Bevy

The project should use the current stable Bevy release rather than its development branch. As of July 2026, Bevy 0.19 is the current major release. Bevy is built around an entity-component-system architecture, and its engine features are composed as plugins. Rendering can be omitted for headless applications, which makes it well suited to running accelerated simulations independently of the visible game.

That combination supports our intended architecture:

```text
Simulation plugins
├── time
├── population
├── households
├── economy
├── agriculture
├── politics
├── warfare
├── religion
├── culture
├── memory
└── historical records

Presentation plugins
├── pixel world rendering
├── animation
├── camera
├── sound
├── interface
├── maps
└── inspectors

Development plugins
├── simulation debugger
├── genealogy viewer
├── event browser
├── economy inspector
├── seed comparison
└── replay tools
```

The project must nevertheless expect Bevy migrations. Bevy remains pre-1.0 and major releases can contain breaking changes. Therefore, engine upgrades should be deliberate development cycles rather than casually performed mid-feature.

---

# 3. The ultimate game fantasy

The fantasy is not simply:

> Rule a kingdom.

Nor is it:

> Survive in a medieval village.

It is:

> Live inside a world that possesses a past, produces a present and is capable of remembering what you do.

The player should be able to discover that:

* the ruined tower outside town belonged to a dynasty destroyed 130 years ago;
* the local spring festival began after a famine;
* two families despise each other because of an inheritance dispute three generations earlier;
* a saint revered in one province is remembered as a rebel elsewhere;
* a folk song describes a battle inaccurately;
* the sword in a noble tomb was forged from metal taken during an actual war;
* the town’s present wealth depends on a bridge built by a disgraced ruler;
* a monster legend began with a misunderstood historical event;
* the player’s grandfather appears in official records very differently from how the family remembers him.

Lore should not primarily be written as encyclopedia entries.

Lore should emerge from:

```text
events
→ witnesses
→ records
→ memories
→ distortions
→ cultural transmission
→ legend
```

This is the project’s defining ambition.

---

# 4. The three layers of the game

## Layer One: The living settlement

The player moves through a pixel-art world at a comprehensible human scale.

They can:

* enter homes;
* speak with inhabitants;
* work;
* trade;
* travel;
* attend ceremonies;
* participate in disputes;
* form relationships;
* join institutions;
* own property;
* establish a household;
* explore ruins;
* serve political or religious causes;
* become involved in war;
* pursue social advancement;
* raise descendants.

This layer gives the simulated history faces, places and emotional weight.

## Layer Two: The kingdom simulation

Behind the visible world, the game simulates:

* households;
* settlements;
* agriculture;
* labor;
* markets;
* transport;
* taxation;
* landholding;
* law;
* dynasties;
* titles;
* succession;
* diplomacy;
* military organization;
* religion;
* migration;
* disease;
* culture;
* literacy;
* knowledge;
* rumor;
* memory.

This creates the pressures that shape daily life.

## Layer Three: Historical memory

The game records and reinterprets what happens.

Events may survive as:

* personal memories;
* household stories;
* legal records;
* chronicles;
* letters;
* songs;
* monuments;
* relics;
* place names;
* religious traditions;
* propaganda;
* legends.

Different sources may contradict one another.

The player should often investigate history rather than receive one authoritative answer.

---

# 5. Core design principles

## History must be causal

A rebellion should not happen because the random-event table selected “rebellion.”

It should arise from conditions such as:

```text
failed harvest
→ falling household reserves
→ inability to pay tax
→ confiscation of land
→ local resentment
→ noble exploitation of grievance
→ royal intervention
→ armed resistance
```

Randomness can influence circumstances, but the result must be explainable.

## People must possess limited knowledge

Characters should not know global game state.

They know what they:

* witnessed;
* were told;
* read;
* inferred;
* remembered;
* or were taught.

This enables rumor, deception, secrecy, mistaken identity, propaganda and historical disagreement.

## Households matter more than isolated individuals

People belong to families, economic units and inherited obligations.

Households should share:

* food;
* property;
* labor;
* debt;
* reputation;
* vulnerability;
* ambitions;
* social connections.

## Institutions persist beyond individuals

Kingdoms, monasteries, guilds, villages and noble houses should possess continuity even as their members change.

## Simulation depth must produce player-facing meaning

A modeled variable should justify itself by influencing:

* a decision;
* a visible consequence;
* another system;
* or the world’s historical character.

Complexity that never reaches the player is engineering compost.

## The world must remain inspectable

Developers and eventually players should be able to understand:

* why a person acted;
* why a price changed;
* why a title transferred;
* why a war began;
* how a rumor spread;
* why a settlement declined;
* what evidence supports a historical claim.

---

# 6. Technical architecture

## Cargo workspace

```text
living_kingdom/
├── crates/
│   ├── merra_core/
│   ├── merra_sim/
│   ├── merra_cli/
│   └── merra_testkit/
├── assets/
├── scenarios/
├── tools/
├── golden/
└── docs/
    ├── devlog/
    └── newsletter/
```

The simulation crates should remain as independent from rendering as practical.
Begin with these broad boundaries and split subsystem crates only after their
interfaces and build costs justify the separation. Era II adds graphical
application and presentation crates without introducing a dependency from the
simulation back to presentation.

## ECS and domain-model hybrid

Use Bevy entities and components for broadly queryable identities:

* people;
* animals;
* households;
* settlements;
* buildings;
* institutions;
* military units;
* titles;
* artifacts.

Use ordinary Rust structures for tightly coupled domain models:

* genealogical indexes;
* property ledgers;
* trade networks;
* succession calculations;
* historical event graphs;
* belief stores;
* legal codes;
* cultural traditions.

Do not decompose every meaningful concept into tiny components merely to worship at the altar of ECS.

## Deterministic simulation

A world should be reproducible from:

* scenario version;
* engine version;
* random seed;
* player commands;
* simulation settings.

Use explicitly managed random streams for separate domains:

```text
weather RNG
birth RNG
mortality RNG
political RNG
combat RNG
name-generation RNG
```

Adding a cosmetic name roll should not unexpectedly change the next year’s harvest.

## Event architecture

Every meaningful change should generate a structured event.

```rust
WorldEvent {
    id,
    time,
    location,
    actors,
    witnesses,
    causes,
    consequences,
    event_type,
    tags,
}
```

Events become the foundation of:

* historical records;
* memories;
* debugging;
* replay;
* newsletter examples;
* chronicles;
* causal explanation.

## Multiple simulation resolutions

Nearby actors receive detailed updates.

Distant actors may be simulated daily, weekly or statistically.

```text
Immediate vicinity
    movement and moment-to-moment actions

Current settlement
    hourly schedules and local interaction

Current region
    daily production and travel

Distant kingdom
    weekly or monthly summaries

Foreign realms
    strategic and demographic abstraction
```

This permits eventual scale without pretending that every peasant on another continent needs frame-by-frame pathfinding.

---

# 7. Development lifecycle

Development will proceed in **cycles**, grouped into larger **eras**.

A cycle should generally represent one coherent question, not an arbitrary time box.

Examples:

* Can people age and die while leaving understandable family histories?
* Can food scarcity alter migration without scripted events?
* Can an inheritance law produce a genuine succession crisis?
* Can rumors spread differently through two neighboring settlements?
* Can a player alter history without becoming the center of every system?

A cycle ends only when it produces:

1. a working result;
2. tests;
3. an inspectable demonstration;
4. a tagged snapshot;
5. one or more reproducible seeds;
6. a complete development record suitable for the Era retrospective.

This protects the project from becoming one giant unfinished branch.

## Standard cycle

### 1. Historical or design question

Begin with one question.

Example:

> What would it take for a minor marriage to cause a war forty years later?

### 2. Model

Define the smallest model capable of investigating it.

### 3. Implementation

Build the system in isolation with fixtures and tests.

### 4. Integration

Connect it to existing systems.

### 5. Simulation runs

Run many seeds and inspect outcomes.

### 6. Player-facing expression

Add the minimum rendering, interface or chronicle output needed to make the result visible.

### 7. Stabilization

Fix invariants, regressions and obvious tuning failures.

### 8. Snapshot

Create:

```text
cycle-007-succession
```

Preserve:

* source version;
* scenario;
* seed;
* output;
* screenshots;
* performance metrics;
* known defects.

### 9. Rusting notes

Complete the cycle record. A standalone public article is optional; the record
must contain enough technical narrative and reproducible evidence to support
the substantial article published at the end of the Era.

### 10. Retrospective

Record:

* what worked;
* what should be removed;
* architectural debt;
* new questions;
* next cycle.

---

# 8. The Rusting newsletter format

Each post should combine lore, engineering and honest development.

## Recommended structure

### Opening chronicle

Begin with the most compelling result.

> King Edric died without a surviving son. His daughter inherited the crown, but the northern lords supported her uncle. The dispute began in Year 37. The marriage that made it possible occurred in Year 4.

### The question

Explain what this cycle attempted to discover.

### What was built

Describe the new system in accessible terms.

### Rust and Bevy

Show selected code, architecture or ECS patterns.

Avoid dumping the entire implementation.

### What emerged

Show two or three seeds with meaningfully different outcomes.

### The causal chain

Explain exactly why the featured event happened.

### What went absurdly wrong

This should become a beloved recurring section.

Examples:

* every noble married the same eighty-two-year-old duchess;
* grain prices became negative;
* a dead bishop continued appointing priests;
* one village accumulated 14,000 geese;
* an infant legally conquered a province;
* the same rumor circled the kingdom for 900 years.

### What changed in the design

Explain what the simulation taught us.

### Next cycle

End with the next question.

## Publication assets

Each cycle should aim to produce at least three of these:

* pixel-art screenshot;
* short GIF;
* kingdom map;
* family tree;
* event timeline;
* chronicle excerpt;
* inspector screenshot;
* code excerpt;
* before-and-after comparison;
* graph of simulation outcomes.

---

# 9. Development Era I: The First Hundred Years

## Goal

Create a headless simulation capable of generating a small but coherent century
of local history inside a world with a longer generated past.

No player character is required yet.

## Cycle 1: Time and mortality

Build:

* deterministic seed;
* fantasy calendar;
* years and seasons;
* generated population;
* aging;
* mortality;
* event log;
* terminal chronicle.

Deliverable:

> Simulate 100 people for 100 years.

Rusting post:

**“I Gave 100 Fictional People a Century to Live”**

## Cycle 2: Families and households

Build:

* parentage;
* marriage;
* childbirth;
* households;
* household formation and dissolution;
* surnames;
* family trees.

Deliverable:

> Inspect a dynasty across four generations.

Rusting post:

**“The First Family Survived. Its Name Did Not.”**

## Cycle 3: Before memory

Build:

* a coarse deterministic world substrate;
* tectonics, elevation, climate, hydrology and biomes;
* resources and prehuman mythic traces;
* one main landmass and a separated island;
* a portable place graph with routes and affordances;
* visual and machine-readable generation evidence.

Deliverable:

> Generate the world before placing historical populations inside it.

## Cycle 4: The first histories

Build:

* aggregate population cohorts;
* separate lineage, culture, faith and polity affiliations;
* one isolated non-human society using shared parameters;
* migration and settlement founding;
* institutions, navigation and route opening;
* contingent first contact and mixed populations;
* competing lore claims and a selected starting region.

Deliverable:

> Run separate human and orc histories until a learned maritime capability
> makes first contact possible.

## Cycle 5: Five villages

Build:

* project the selected macro-history region into detailed settlements;
* graph-based roads and travel times;
* settlement populations reconciled with aggregate cohorts;
* local migration;
* births and deaths by place;
* historical institutions and claims visible at household scale.

Deliverable:

> Enter five villages whose differences have world-scale causes.

## Cycle 6: Food and survival

Build:

* farmland;
* labor;
* seasonal harvest;
* household food reserves;
* hunger;
* storage losses;
* famine migration;
* mortality consequences.

Deliverable:

> A failed harvest visibly alters household and settlement history.

## Cycle 7: Property and inheritance

Build:

* land;
* homes;
* tools;
* money;
* ownership;
* inheritance rules;
* debts.

Deliverable:

> Property changes hands through death, marriage and debt.

## Cycle 8: Titles and nobility

Build:

* lords;
* titles;
* vassal relationships;
* succession law;
* claims;
* legitimacy.

Deliverable:

> Generate the first unscripted succession dispute.

## Cycle 9: The disputed heir

Build:

* competing claims;
* political factions;
* inherited obligations;
* legitimacy disputes;
* mobilization and negotiated settlement;
* the first understandable political crisis.

Deliverable:

> Trace one conflict from household history through institutions to a disputed
> succession.

## Era I completion criteria

The simulation can produce a readable hundred-year chronicle containing:

* births;
* marriages;
* deaths;
* migrations;
* famines;
* household changes;
* property transfers;
* dynastic succession;
* at least one understandable political crisis.

---

# 10. Development Era II: The Visible Village

## Goal

Transform the headless simulation into a small pixel-art world.

## Broad features

* fixed-resolution pixel presentation;
* one explorable settlement;
* tile-based terrain;
* roads, fields, homes and workshops;
* day and night;
* seasons;
* weather;
* animated inhabitants;
* basic schedules;
* object interaction;
* world inspection;
* time controls.

The initial visual objective is not an enormous map. It is to make one simulated village feel inhabited.

## Key cycles

### Pixel rendering foundation

* camera;
* integer scaling;
* sprite atlas handling;
* tile rendering;
* depth sorting;
* animation states.

### Settlement projection

Convert simulation entities into visible places and characters.

### Daily schedules

Residents:

* wake;
* eat;
* work;
* travel;
* socialize;
* return home;
* sleep.

### Player observation mode

The player initially acts as an observer or chronicler.

They can:

* pause;
* accelerate time;
* click characters;
* inspect households;
* follow people;
* review events.

### Historical overlays

Add:

* ownership view;
* family view;
* food-security view;
* political allegiance view;
* settlement history.

## Era II completion criteria

A player can observe one village for a year and understand:

* who lives there;
* what they do;
* which households are thriving;
* who is related;
* where food comes from;
* how a death or failed harvest changes visible life.

---

# 11. Development Era III: The Social World

## Goal

Make people socially and psychologically distinct enough to create memorable interpersonal history.

## Broad features

* traits;
* needs;
* goals;
* affection;
* trust;
* respect;
* resentment;
* fear;
* dependence;
* obligation;
* memory;
* limited knowledge;
* rumor;
* promises;
* favors;
* reputation.

## Major systems

### Memory

Characters retain significant events.

Different witnesses interpret the same event differently.

### Belief

Characters hold claims with varying confidence.

```text
“The miller caused the shortage.”
Confidence: 0.62
Source: heard from cousin
```

### Rumor

Information moves through relationships and geography.

### Social obligations

Characters remember:

* promises;
* debts;
* hospitality;
* insults;
* aid;
* failures of reciprocity.

### Reputation

Reputation is not one universal number.

It is the distributed result of what different groups believe.

## Era III completion criteria

The player can follow a conflict between two households and trace:

* its original cause;
* who knows what;
* how stories diverged;
* which memories sustain it;
* how it affects later decisions.

---

# 12. Development Era IV: Economy and Material Civilization

## Goal

Build an economy that generates historical pressure rather than merely producing shop prices.

## Broad features

* professions;
* labor;
* production chains;
* tools;
* workshops;
* markets;
* contracts;
* credit;
* transport;
* taxation;
* land rents;
* trade routes;
* shortages;
* substitution;
* wealth;
* social mobility.

## Possible production chains

```text
forest
→ timber
→ sawmill
→ boards
→ buildings and ships
```

```text
sheep
→ wool
→ spinning
→ yarn
→ weaving
→ cloth
→ clothing
```

```text
ore
→ smelting
→ metal
→ tools and weapons
```

The economy should make institutions and political decisions materially consequential.

## Era IV completion criteria

The simulation can explain:

* why a settlement became wealthy;
* why a trade route changed;
* why a household fell into debt;
* why a shortage occurred;
* who benefited from a tax;
* how economic disruption contributed to political conflict.

---

# 13. Development Era V: Kingdoms and Governance

## Goal

Create political institutions that possess continuity, internal conflict and material power.

## Broad features

* law;
* offices;
* councils;
* noble factions;
* royal courts;
* legitimacy;
* taxation authority;
* land grants;
* vassalage;
* diplomacy;
* treaties;
* succession;
* rebellions;
* administrative capacity;
* corruption;
* justice.

Political action should be constrained by:

* information;
* money;
* loyalty;
* geography;
* law;
* custom;
* personal relationships;
* military strength;
* institutional legitimacy.

## Player possibilities

The player may eventually:

* petition a lord;
* serve in an office;
* become a vassal;
* collect taxes;
* arbitrate disputes;
* influence succession;
* found a house;
* rule a settlement;
* take the crown;
* reject power entirely.

## Era V completion criteria

A kingdom can endure a disputed succession, with factions making understandable choices based on claims, relationships, law and self-interest.

---

# 14. Development Era VI: War as a Social Catastrophe

## Goal

Model war as more than armies colliding.

## Broad features

* levies;
* professional soldiers;
* commanders;
* supply;
* morale;
* terrain;
* travel;
* sieges;
* raiding;
* casualties;
* desertion;
* prisoners;
* occupation;
* refugees;
* taxation;
* food requisition;
* veteran return;
* political consequences.

Tactical battles may eventually become visible, but the first priority is the historical system around warfare.

A battle should affect:

* households that lose labor;
* fields left unharvested;
* inheritance;
* local prices;
* migration;
* disease;
* memory;
* songs;
* legitimacy;
* later politics.

## Era VI completion criteria

The consequences of a war remain visible for a generation after the fighting ends.

---

# 15. Development Era VII: Religion, Culture and Meaning

## Goal

Allow societies to interpret the world, not merely occupy it.

## Broad features

* religions;
* sacred places;
* clergy;
* rituals;
* doctrines;
* heresies;
* pilgrimages;
* moral authority;
* festivals;
* taboos;
* education;
* literacy;
* languages;
* songs;
* stories;
* artistic traditions;
* funerary customs;
* calendars;
* omens.

Religious and cultural systems should influence:

* marriage;
* legitimacy;
* law;
* warfare;
* identity;
* food;
* burial;
* memory;
* diplomacy.

## Era VII completion criteria

Two regions can remember the same ruler or war in fundamentally different cultural terms.

---

# 16. Development Era VIII: Generated Lore and Historical Memory

## Goal

Turn simulation history into layered, contradictory lore.

## Broad features

* written chronicles;
* oral histories;
* monuments;
* relics;
* songs;
* genealogies;
* myths;
* place names;
* historical schools;
* censorship;
* forgery;
* lost archives;
* rediscovery;
* propaganda.

## Historical transformation pipeline

```text
world event
→ witnessed experience
→ personal memory
→ retelling
→ record
→ copying
→ political reinterpretation
→ folklore
```

The engine should preserve the underlying event while allowing historical representations to diverge.

## Example

Actual event:

> A local commander retreated because food supplies failed.

Royal chronicle:

> The commander abandoned the field through cowardice.

Local song:

> He saved his soldiers from a doomed battle.

Enemy history:

> Their army broke before our advance.

Family tradition:

> He was betrayed by the crown.

Centuries later, the player may encounter all four.

## Era VIII completion criteria

The player can investigate a historical event through contradictory evidence and form their own conclusion.

---

# 17. Development Era IX: The Player Enters History

## Goal

Move from observer simulation to lived role-playing.

## Broad features

* character creation;
* background;
* household membership;
* skills;
* work;
* social interaction;
* reputation;
* property;
* travel;
* injury;
* aging;
* marriage;
* children;
* succession to descendants;
* institutional membership;
* player death.

The game should not necessarily end when the first player character dies.

The player may continue as:

* a child;
* a relative;
* an apprentice;
* an institutional successor;
* another person influenced by the previous life.

This allows gameplay to span generations.

## Player verbs

* observe;
* speak;
* promise;
* lie;
* trade;
* work;
* study;
* travel;
* fight;
* serve;
* persuade;
* testify;
* worship;
* build;
* inherit;
* teach;
* record;
* rule.

## Era IX completion criteria

A player can live one entire life and leave consequences that remain after death.

---

# 18. Development Era X: The Larger World

## Goal

Scale from one kingdom to a world of interacting societies.

## Broad features

* multiple kingdoms;
* foreign cultures;
* long-distance trade;
* migration;
* diplomacy;
* imperial expansion;
* borderlands;
* exploration;
* climate regions;
* technological diffusion;
* religious exchange;
* colonization;
* collapse;
* diaspora.

Foreign regions should initially operate at lower simulation resolution and become detailed when the player approaches or when events make them important.

## Era X completion criteria

A local player can experience consequences produced by distant events:

* war interrupts trade;
* refugees arrive;
* a foreign religion spreads;
* imported technology changes production;
* a distant succession alters local politics.

---

# 19. Development Era XI and beyond

This project should deliberately leave room for future eras that cannot yet be completely planned.

Possible directions include:

* ecology and animal populations;
* naval trade and exploration;
* architecture that evolves historically;
* procedural dialects and language drift;
* schools of philosophy;
* medicine and epidemic knowledge;
* legal precedent;
* technological invention;
* historical archaeology;
* magical traditions;
* divine intervention;
* underground civilizations;
* simulation of dreams, prophecy and myth;
* modding and scenario creation;
* multiplayer chronicles;
* standalone world-generation tools;
* simulation research APIs.

The project is allowed to grow for years, but each addition must deepen the central idea:

> A world creates history, remembers it imperfectly and allows the player to live inside the result.

---

# 20. Pixel-art strategy

The visual style should support scale and readability rather than chase extravagant animation counts.

## Initial direction

* top-down or high three-quarter perspective;
* restrained tile size;
* crisp integer scaling;
* limited seasonal palettes;
* compact character silhouettes;
* modular clothing and heraldry;
* strong building profiles;
* visible settlement growth.

## Visual development order

1. terrain and camera;
2. settlement blocks;
3. inhabitants;
4. seasons and weather;
5. interiors;
6. work animations;
7. heraldry and institutions;
8. battle and travel;
9. regional maps;
10. historical changes to architecture.

The world should visibly age.

Buildings can:

* receive additions;
* decay;
* burn;
* be rebuilt;
* change ownership;
* acquire plaques;
* become ruins;
* become sacred or politically significant.

A screenshot taken two simulated centuries apart should reveal that history occurred.

---

# 21. Tooling roadmap

The project’s internal tools may become nearly as ambitious as the game.

## World inspector

Inspect any:

* person;
* household;
* settlement;
* institution;
* title;
* artifact;
* event.

## Causal explorer

Ask:

> Why did this happen?

Display the chain of relevant events and conditions.

## Genealogy viewer

Explore family relationships over centuries.

## Political map

Show:

* territory;
* claims;
* allegiance;
* control;
* disputed regions.

## Economic flow viewer

Trace:

* production;
* trade;
* scarcity;
* taxation;
* household consumption.

## Historical source viewer

Compare:

* actual simulation events;
* witness memories;
* official records;
* folklore.

## Seed laboratory

Run hundreds or thousands of worlds and compare:

* population;
* dynasty survival;
* wars;
* famine;
* wealth concentration;
* settlement growth;
* institutional longevity.

## Replay system

Reconstruct history from commands, seeds and event records.

These tools will also produce excellent Rusting newsletter visuals.

---

# 22. Testing strategy

## Unit tests

Test domain rules:

* inheritance;
* food consumption;
* title eligibility;
* travel duration;
* relationship changes.

## Invariant tests

Examples:

* dead characters cannot initiate actions;
* children have valid parents where recorded;
* property has one legal owner;
* food cannot be consumed twice;
* a letter cannot arrive before being sent;
* a title holder must satisfy the title’s current eligibility law.

## Property-based tests

Generate unusual families, economies and succession structures to uncover edge cases.

## Statistical simulation tests

Run many seeds and define acceptable ranges rather than exact outcomes.

## Golden seeds

Maintain a collection of interesting worlds:

```text
seed_0042_stable_dynasty
seed_1711_famine_migration
seed_2884_child_queen
seed_9012_monastic_empire
```

## Narrative quality tests

Some tests will remain qualitative:

* Is the cause understandable?
* Did the outcome feel repetitive?
* Did the world expose the event clearly enough?
* Did history produce recognizable continuity?
* Did an apparently dramatic result actually matter?

---

# 23. Release philosophy

Do not hide the project until it is “ready.”

Release development artifacts in layers.

## Internal snapshots

Every cycle.

## Public source snapshots

Potentially selected cycles or crates.

## Playable laboratories

Small downloadable experiments:

* population simulator;
* genealogy generator;
* village observer;
* succession sandbox;
* rumor simulator;
* medieval economy toy.

## Major public milestones

* first generated century;
* first visible village;
* first player interaction;
* first succession crisis;
* first war;
* first multigenerational playthrough;
* first contradictory historical record;
* first world containing multiple kingdoms.

These can sustain years of public progress without pretending the final game is around the corner.

---

# 24. Scope discipline for an extremely ambitious project

Extreme ambition does not mean building everything simultaneously.

It means preserving a huge destination while advancing through complete, durable layers.

Each cycle must avoid:

* replacing working systems without strong reason;
* beginning three unrelated major features;
* simulating the whole world's actors at local resolution before local depth
  works;
* adding lore unconnected to simulation;
* polishing graphics to conceal shallow systems;
* adding AI where deterministic rules are better;
* chasing every new Bevy plugin;
* rewriting the architecture as a form of procrastination.

The rule is:

> Increase the world’s depth one causal relationship at a time.

---

# 25. The first fourteen cycles

A concrete opening sequence:

1. **Time and Death**
   Calendar, population, aging, mortality and chronicle.

2. **The Household**
   Families, marriage, childbirth and household continuity.

3. **Before Memory**
   Physical context, resources, mythic traces and a portable place graph.

4. **The First Histories**
   Separate peoples, migration, institutions, navigation and first contact.

5. **Five Villages**
   Places, roads, migration and settlement identity.

6. **The Harvest**
   Food, labor, weather and hunger.

7. **What the Dead Leave Behind**
   Property, debt and inheritance.

8. **The First Crown**
   Titles, rulers, legitimacy and succession.

9. **The Disputed Heir**
   Claims, factions and the first political conflict.

10. **A Village You Can See**
   Pixel rendering of one simulated settlement.

11. **A Day in Dunmere**
   Visible routines and schedules.

12. **What People Remember**
    Personal memory and relationship consequences.

13. **The Rumor Crosses the River**
    Limited knowledge and information propagation.

14. **The Chronicle Lies**
    Divergence between actual events and recorded history.

At the end of these cycles, the project should already possess a distinctive identity.

It will not yet be the complete game.

It will be a visible, simulated medieval society that lives, dies, inherits, struggles, remembers and misremembers.

That is the foundation on which the next several years can stand.

---

# 26. The north-star demonstration

The ultimate demonstration of the project should be something like this:

A player arrives in a prosperous market town.

They learn that the town’s annual feast honors Saint Merra, who supposedly saved the valley from an invading king.

The monastery chronicle says Merra led the townspeople in resistance.

A noble family claims its ancestor fought beside her.

A neighboring region remembers Merra as a bandit.

The player discovers an older tax ledger, a ruined bridge and letters preserved by a merchant household.

The actual simulation history reveals that:

* Merra was a miller;
* the royal army never intended to invade;
* a tax dispute became violent;
* the bridge collapsed during the confrontation;
* food stores were destroyed;
* Merra organized relief afterward;
* later rulers converted the event into a patriotic legend.

No writer manually authored every stage of that mythology.

The simulation created the event.

People remembered it differently.

Institutions preserved useful versions.

Time transformed it into legend.

The player encountered the legend as part of ordinary life.

That is the game we are setting out to build.

Not merely a kingdom generator.

Not merely a pixel role-playing game.

A machine for producing history—and a place where the player can live inside it.
