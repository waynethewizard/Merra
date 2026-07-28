# Design Principles

> **Status:** Foundational
> **Purpose:** Guide simulation design, implementation choices, demo scope, and future expansion.
> **Applies to:** World generation, historical simulation, cultures, households, institutions, politics, memory, and player-facing investigation.

## Design thesis

The project should generate histories that feel authored without being prewritten.

It does this by simulating people, households, institutions, resources, obligations, beliefs, and information over time, then allowing meaningful differences to emerge from how those shared systems are configured and transformed.

The engine is not “a medieval simulator,” “an orc simulator,” or even necessarily “a fantasy simulator.” Those are expressions of a deeper historical simulation. The same foundations should eventually be capable of producing a feudal kingdom, a Victorian industrial region, a frontier colony, or a far-future diaspora without replacing the core machinery.

---

## 1. Simulate causes, not outcomes

Do not begin by deciding that a kingdom collapses, a people become warlike, or a dynasty is cursed. Simulate the pressures and decisions that could make those outcomes occur.

Prefer:

- food shortages that strain obligations;
- inheritance rules that fragment estates;
- rumors that turn uncertainty into panic;
- institutions that preserve old grievances;
- rulers whose choices alter legitimacy;
- households that remember losses across generations.

Avoid scripts that manufacture historical drama without causal support. Authored events may introduce pressure, but the simulation should determine what that pressure means.

**Decision test:** Can the result be explained as a chain of system states and agent decisions?

---

## 2. Shared machinery, different parameters

Cultures, peoples, species, factions, and eras should usually be expressions of the same systems—not parallel implementations.

An orc clan and a human duchy may both use the same systems for households, reputation, information, inheritance, and obligation. Their differences should arise from parameter bundles, institutional forms, norms, and historical circumstances.

For example, a rumor system may already contain:

- transmission rate;
- distortion rate;
- trust weighting;
- channel restrictions;
- salience;
- decay;
- institutional preservation.

A ritualized oral culture might transmit information through fewer channels and therefore spread news more slowly, while preserving wording and provenance unusually well. A cosmopolitan trading culture might spread news rapidly but introduce far more mutation. Centuries later, an oral account may be more reliable than an official written chronicle—not because the game declared one culture truthful, but because the same information system produced different historical records under different conditions.

**Decision test:** Is this difference a new system, or can it be represented as a new configuration of an existing one?

---

## 3. Systems must allow for plurality and drift

No society should be reducible to one permanent cultural setting.

Culture is multi-layered, internally contested, geographically uneven, and historically unstable. Beliefs, customs, institutions, identities, and political loyalties should be capable of changing at different speeds.

The simulation should allow:

- several cultures within one polity;
- several identities within one person or household;
- regional and class variation;
- minority practices that persist despite official norms;
- syncretism, assimilation, revival, and schism;
- institutions that preserve customs individuals no longer understand;
- younger generations that reinterpret inherited obligations;
- settings that drift because of events, incentives, and contact.

A culture definition is an initial condition and a set of pressures—not a timeless essence.

**Decision test:** Can this identity divide, merge, migrate, weaken, revive, or change meaning over time?

---

## 4. Culture is not biology, and lineage is not destiny

Fantasy peoples may have biological or supernatural differences, but behavior should not be hardwired into ancestry unless the fiction absolutely requires it.

Do not encode “orcs are warlike,” “elves are wise,” or “humans are ambitious” as behavioral truth. Encode the institutions, lifespans, environments, historical traumas, material constraints, and social incentives that could make particular tendencies emerge.

A long-lived people may accumulate different forms of institutional memory. A people with dangerous childbirth may organize households differently. A magically altered lineage may experience inheritance in unusual ways. These differences can matter without becoming moral or cultural destiny.

Mixed settlements, migration, adoption, conversion, intermarriage, and multi-lineage households should be considered in the foundations even if they are not present in the first playable cycles.

**Decision test:** Would a child raised elsewhere necessarily behave according to ancestry? If yes, the design should have a strong worldbuilding reason.

---

## 5. Institutions are historical actors

Individuals die. Institutions preserve, distort, classify, and weaponize memory.

Clans, temples, courts, guilds, archives, armies, councils, schools, newspapers, and bureaucracies should carry state across generations. They should have interests, procedures, resources, reputations, and inherited relationships.

Institutions may:

- preserve an oath after every original participant is dead;
- keep records that privilege one version of an event;
- maintain a feud whose original cause has been forgotten;
- outlive the culture that founded them;
- resist or accelerate cultural change;
- translate personal memories into official history;
- lose knowledge through destruction, neglect, secrecy, or reform.

History should not exist only inside individual agents.

**Decision test:** What continues to exert pressure after the people involved are gone?

---

## 6. Obligations are first-class simulation objects

Promises, contracts, debts, favors, fealty, kinship duties, blood oaths, legal judgments, and religious vows should use a shared obligation model with configurable behavior.

An obligation may have:

- parties and beneficiaries;
- witnesses or enforcing institutions;
- scope and required action;
- legitimacy in different communities;
- strength and salience;
- decay or expiration;
- inheritance rules;
- conditions for transfer;
- penalties for breach;
- mechanisms for release, renegotiation, or fulfillment.

A blood oath can therefore be a distinct class of obligation without requiring an entirely separate political system: near-zero decay, strong descendant inheritance, high social salience, and catastrophic reputational consequences for breach.

Human treaties may be easier to abandon but rely on courts, hostages, trade sanctions, or military enforcement. Cross-cultural politics then emerges from incompatible assumptions about what makes a promise binding.

**Decision test:** Can a new form of duty be represented by changing the lifecycle and enforcement of an obligation?

---

## 7. Cross-cultural mismatch is a source of history

Difference becomes interesting when systems interact.

The goal is not merely to create several isolated cultures with distinct bonuses. The simulation should produce consequences when groups disagree about:

- who may speak for a community;
- what counts as evidence;
- whether a promise binds descendants;
- how property passes between generations;
- whether land can be owned, used, or held in trust;
- what constitutes an insult or restitution;
- how quickly news is trusted;
- who has authority to forgive a debt;
- whether written, witnessed, ritualized, or remembered agreements are legitimate.

A ruler who understands another society’s institutions can cooperate with or exploit them. A ruler who assumes everyone shares the same model of law, kinship, or truth can create a conflict that lasts for centuries.

**Decision test:** Does contact create new pressures and misunderstandings, or only exchange cosmetic traits?

---

## 8. Households are the bridge between person and civilization

The household should be a core unit of continuity. It connects intimate life to economics, inheritance, politics, migration, labor, and memory.

Households may include biological relatives, spouses, adoptees, dependents, servants, apprentices, hostages, lodgers, ritual kin, or other culturally recognized members. Household structure should be configurable rather than assumed to be a universal nuclear family.

The foundations should anticipate:

- mixed-culture and mixed-lineage households;
- competing inheritance traditions;
- conversion or assimilation within a household;
- marriage as alliance, affection, coercion, or economic arrangement;
- adoption and fosterage;
- household division and reunification;
- descendants inheriting obligations from several traditions.

Do not postpone every difficult household question until multiple cultures arrive. The implementation may begin simply, but the data model should not make plurality impossible.

**Decision test:** Can one household contain people governed by overlapping customs without duplicating household code?

---

## 9. Information has provenance, mutation, and survival

The simulation should distinguish an event from the claims later made about it.

Information should move through people and institutions, accumulating:

- sources;
- witnesses;
- copying and retelling chains;
- distortion;
- omissions;
- confidence;
- political incentives;
- preservation media;
- destruction risks;
- cultural trust.

A written record is not automatically true. An oral account is not automatically vague. An official archive may be precise but propagandistic. A family story may mutate yet preserve a fact the state erased.

This principle supports a future player experience built around historical reconstruction: the world contains not only a past, but evidence about the past.

**Decision test:** Can the player ask both “what happened?” and “why does this source say that happened?”

---

## 10. Memory should be uneven

The world should forget selectively.

High-salience events may survive for centuries while ordinary but consequential events disappear. Different communities may preserve different parts of the same episode. Institutions may remember procedures but forget purposes. Names may survive after meanings are lost.

Forgetting is not merely deleted data. It can become:

- myth;
- ritual without explanation;
- a disputed boundary;
- an inherited taboo;
- a corrupted title;
- a ruin with several competing identities;
- a debt no one can fully justify;
- a political claim built on partial evidence.

The ruins and mysteries of the present should often be the compressed residue of earlier simulation.

**Decision test:** What trace remains after the detailed state is gone?

---

## 11. Settings are expressions, not the engine

Do not bind the deepest systems to one genre, map, or historical aesthetic.

A feudal oath, a Victorian labor contract, and a spacefaring service covenant may all be obligations. A clan moot, a newspaper network, and a communications relay may all be information channels. A noble house, an urban tenement, and a generation ship family may all be households.

The initial fantasy-kingdom setting should make the systems vivid, but implementation language should remain conceptually portable where doing so does not harm clarity.

This does not mean making every system abstract for abstraction’s sake. Build the current game well. Avoid needless assumptions that make the next setting impossible.

**Decision test:** Is this concept inherently medieval, or are we giving a general historical process a medieval presentation?

---

## 12. Data defines variation; code defines capability

Parameters, rules, and content should be data-driven wherever practical.

Code should answer questions such as:

- Can obligations decay?
- Can a household split?
- Can information mutate?
- Can institutions inherit claims?
- Can identities overlap?

Data should answer questions such as:

- How quickly does this obligation decay?
- Who inherits it?
- Which channels does this community trust?
- How is this household structured?
- What makes this event culturally salient?

Parameter bundles should be inspectable, composable, versioned, and testable. Avoid scattering cultural assumptions across agent logic.

**Decision test:** Could a designer create a substantially different society without changing Rust code?

---

## 13. Parameters are pressures, not personality labels

Avoid replacing hardcoded stereotypes with a spreadsheet of stereotypes.

A value such as “honor = 0.9” is too broad to explain behavior. Prefer specific mechanisms: oath breach has high salience; witnessed promises transmit strongly; compensation restores reputation poorly; descendants inherit unresolved obligations.

Specific parameters create behavior that can be understood and challenged. Vague cultural traits create essentialism and unpredictable interactions.

**Decision test:** Does each parameter correspond to an observable process in the simulation?

---

## 14. Emergence must remain legible

A surprising outcome is only valuable when the player or developer can eventually understand it.

Important state changes should preserve enough causal information to support:

- debugging;
- simulation inspection;
- historical summaries;
- in-world records;
- player investigation;
- development posts explaining what happened.

The project should be able to answer questions such as:

- Why did this war begin?
- Why did this family lose its estate?
- Why is this oath still active?
- Why does one community believe a different history?
- Which event caused this institution to radicalize?

Do not confuse opacity with depth.

**Decision test:** Can the simulation produce a human-readable causal account of this outcome?

---

## 15. Determinism is a creative tool

Given the same seed, version, and inputs, a simulation run should be reproducible unless nondeterminism is an explicit feature.

Reproducibility enables:

- debugging emergent chains;
- comparing parameter changes;
- regression testing;
- replaying notable histories;
- writing credible development posts;
- saving and sharing world seeds.

Snapshots should include the simulation version and relevant configuration so that old worlds remain interpretable even after the engine changes.

**Decision test:** Can a notable history be replayed and examined rather than merely remembered?

---

## 16. Begin with a legible baseline

Build and understand one baseline society before adding several peoples, cultures, or eras.

The initial human—or otherwise chosen—baseline should prove that households, resources, information, obligations, institutions, and succession can generate comprehensible history. A second major culture should arrive only when the shared systems are mature enough to express meaningful difference.

The code should be plurality-ready from the beginning, but the content should expand deliberately.

A useful later cycle might introduce “the clans across the hills”: the same simulation code, configured around different household forms, oral transmission, oath inheritance, and political authority. The point of the cycle would be to demonstrate that the engine can produce another civilization without adding another engine.

**Decision test:** Are we adding a new people because the systems are ready to reveal something, or because the baseline is not yet interesting?

---

## 17. Every development cycle must produce evidence

Each round should end with something observable, explainable, and preservable.

A cycle should ideally produce:

1. a working vertical slice or simulation capability;
2. one or more reproducible example histories;
3. an inspectable visualization, report, scene, timeline, or artifact;
4. tests or diagnostics that protect the capability;
5. a snapshot of the project;
6. a development post explaining the question, implementation, result, surprise, and next step.

Do not spend several cycles building invisible infrastructure without creating a visible consequence. Internal architecture matters, but each round should prove why it matters.

**Decision test:** What can another person see, run, or read at the end of this cycle?

---

## 18. Build narrow slices through the whole simulation

Prefer a small complete chain over a broad collection of disconnected subsystems.

For example:

> a harvest failure changes household stores → a household calls in a debt → the debtor breaks an obligation → witnesses spread the breach → reputation changes → a marriage negotiation fails → the resulting grievance appears in the next generation.

That narrow chain teaches more about the engine than separately building an elaborate economy, rumor system, genealogy system, and political system that do not yet affect one another.

**Decision test:** Does this feature participate in a causal loop, or merely exist beside other features?

---

## 19. Complexity must earn its place on screen

The simulation may be deep, but every layer should eventually create a visible difference in histories, decisions, evidence, or presentation.

Do not model details solely because they are historically plausible. Model them when they:

- alter choices;
- create consequences;
- explain variation;
- support investigation;
- produce evocative artifacts;
- or unlock future systems at reasonable cost.

Aggregate or compress details that do not yet matter. Preserve extension points without simulating the universe in Era I.

**Decision test:** What player-visible or developer-visible difference does this complexity produce?

---

## 20. Preserve surprise without surrendering authorship

The simulation should surprise its creators, but the project still needs a point of view.

Themes should emerge from the systems selected, the pressures emphasized, the evidence shown, and the questions the player is invited to ask. Themes need not be imposed through fixed plots.

Possible recurring themes include:

- the burden of inheritance;
- the unreliability of official history;
- the distance between law and legitimacy;
- the way institutions outlive their purposes;
- the cost of promises made for descendants;
- the transformation of memory into myth;
- the danger of treating another society’s customs as inferior versions of one’s own.

The engine generates events. The design chooses what kinds of causes, losses, loyalties, and memories are worth simulating.

**Decision test:** What human question does this system make the generated history capable of asking?

---

# Practical model: capability, configuration, state

When designing a feature, separate three layers:

### Capability

What the engine permits.

Examples: obligations can be inherited; rumors can mutate; households can split; institutions can preserve records.

### Configuration

How a particular culture, institution, era, or scenario uses that capability.

Examples: blood oaths rarely decay; a royal archive trusts sealed testimony; a clan recognizes foster siblings as heirs.

### State

What has actually happened in this world.

Examples: this oath has passed through four generations; this archive burned; this household adopted a refugee; this account became distorted after six retellings.

Keeping these layers separate prevents current content from becoming permanent engine law.

---

# Example: one engine, different expressions

| Shared system | Human kingdom expression | Hill-clan expression | Emergent consequence |
|---|---|---|---|
| Information | Fast informal gossip; written records carry official prestige | Slower formal retelling; oral provenance carries prestige | Official chronicles spread quickly, but clan testimony may remain more accurate |
| Obligation | Contracts decay or expire; courts and enforcement matter | Oaths decay slowly and may bind descendants | Each side misjudges what the other believes a treaty means |
| Household | Property-centered inheritance with strong legal recognition | Kinship and fosterage-centered inheritance | Adoption or marriage changes political succession in unexpected ways |
| Reputation | Repairable through compensation, office, or legal judgment | Breach remains highly salient in kin networks | A legally settled dispute remains socially active for generations |
| Memory | Archives centralize and standardize accounts | Ritual specialists preserve selected narratives | Destroying an archive harms one society more; killing memory-keepers harms the other |

These should remain examples, not permanent definitions. Both societies must be capable of internal diversity and historical change.

---

# Scope rules for early eras

1. **Era I should prove the baseline causal loop.** Do not introduce several major peoples merely to demonstrate extensibility.
2. **Make key values configurable immediately.** Transmission, distortion, trust, decay, inheritance, household membership, and institutional memory should not be buried as constants.
3. **Do not build unused generality.** Add the smallest extension points that keep later plurality possible.
4. **Test parameter extremes.** Before adding a second culture, verify that the baseline systems remain stable under slow and fast transmission, weak and strong obligations, and different household rules.
5. **Introduce major variation as a dedicated cycle.** Give a new society enough attention to expose weaknesses in the shared engine rather than slipping it in as flavor content.
6. **Treat mixed cases as architectural tests.** Even before fully supporting intermarriage or mixed settlements, use them as thought experiments against the data model.

---

# Warning signs

Reconsider a design when:

- adding a culture requires copying an existing system;
- a behavioral rule is attached directly to ancestry;
- a society is represented by one global scalar;
- every member of a culture behaves identically;
- cultural parameters never change during a run;
- institutions are only labels on individual agents;
- written records are treated as objective truth;
- history is stored only as an omniscient event log;
- a feature cannot affect any other system;
- a surprising result cannot be explained;
- a development cycle ends with nothing visible;
- genre-specific terminology has leaked into the deepest engine layer without need.

---

# Review checklist

Before accepting a major system or content addition, ask:

- Does it create causes rather than prescribe outcomes?
- Does it reuse shared machinery?
- Is its variation data-driven and inspectable?
- Can it support internal plurality?
- Can it drift over time?
- Can institutions carry it across generations?
- Can mixed households or mixed communities exist within the model?
- Does it avoid turning ancestry into destiny?
- Does it interact with at least one existing system?
- Can its consequences become historical evidence?
- Can the result be explained to a player or reader?
- Can it be reproduced from a seed and version?
- Does the current cycle produce something visible?
- Would the principle still make sense in another setting?

---

# North star

**Build a simulation in which peoples differ because their histories, institutions, environments, and inherited practices shape how shared human—or sentient—systems operate. Let those differences interact, drift, and leave evidence. Never mistake the starting configuration for the nature of a people, and never mistake the setting for the engine.**
