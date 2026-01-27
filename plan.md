# Cross-Fork (echter Fork): **Mozilla Readability** × **Defuddle** — 1:1 Port nach Rust + Node.js Bindings

Stand: 2026-01-27  
Zielbild: **Algorithmisch 1:1** (so weit praktikabel) Port von Readability (Step 1) und danach **Defuddle-Deltas** (Step 2) — jeweils inkl. **Upstream-Tests/Fixtures**, plus ein modernes Tooling-Setup (TS strict, typed-eslint strict, Prettier, Vitest).

---

## 0) Was „Fork“ hier bedeutet (und warum das wichtig ist)

Du meinst „Fork“ im Git-Sinne + „1:1 Konvertierung“ im Implementations-Sinne:

1. **Git-Traceability & Lizenz-Compliance:** Upstream-Quellen bleiben nachvollziehbar (Commit-SHAs, LICENSE/NOTICE).  
2. **Transliteration statt Re-Design:** Rust-Code folgt Struktur/Flow/Heuristiken der Originale so eng wie möglich.  
3. **Tests als Contract:** Upstream-Tests/Fixtures werden *mitgezogen* und dienen als objektiver Referenzpunkt.

> Ergebnis: Ein Repo, das „forkt“ (Upstream als Remote/Subtree) und parallel den Rust-Port aufbaut, ohne den Bezug zu verlieren.

---

## 1) Upstream-Testlage: was du „mitnehmen“ kannst

### Readability
Readability enthält eine umfangreiche Fixture-Suite unter `test/test-pages/*` (pro Case typischerweise: `source.html` + `expected.html` + Metadaten). Beispiel: `001/source.html` und `001/expected.html`.  
Außerdem beschreibt Readabilitys Contribution-Guide eine `generate-testcase`-Pipeline zum Erzeugen neuer Fixture-Cases (inkl. expected output).  

**Konsequenz für deinen Port:**  
- Diese Fixtures werden *1:1* in dein Repo gespiegelt.  
- Deine Rust-Implementierung wird gegen genau diese Fixtures getestet (golden tests).  
- Zusätzlich kann ein „Oracle-Test“ laufen, der die JS-Referenz (Upstream) für dieselbe `source.html` ausführt und das Rust-Resultat bit-/diff-nah vergleicht.

### Defuddle
Defuddle führt im Repo eine `tests/`-Struktur. (Details variieren je Version; wir behandeln das im Plan als zweite Portierungswelle.)  

**Konsequenz für deinen Port:**  
- Defuddle-Tests/Fixtures werden ebenfalls gespiegelt.  
- Sobald Readability-Port stabil ist, kommt Defuddle als Delta-Layer oben drauf.

---

## 2) Ziel-API (Node) und Kern-Model (Rust)

### 2.1 Rust Core: zwei Schichten
**A) DOM/Tree-Layer (Shim)**
- Ziel: eine minimalistische DOM-API, die **Readability/Defuddle-Operationen** abdeckt.
- Gründe: 1:1 Port wird nur praktikabel, wenn du JS-DOM-Calls auf stabile Rust-Primitives mappst.

**B) Algorithmus-Layer**
- `readability`-Modul: Port von `Readability.js` und ggf. `Readability-readerable.js`.
- `defuddle`-Modul: Port der Defuddle-Logik als „Patchset“/Delta gegen Readability oder als paralleler Extractor.

### 2.2 Node Bindings
- Empfehlung: **napi-rs** (N-API) für stabile Node ABI + gute TS-Story.
- `@your-scope/readability-defuddle` (oder zwei Packages) als Distribution.

#### Beispiel-API (TS)
```ts
export interface ExtractOptions {
  url?: string;
  debug?: boolean;
  // … (1:1 soweit sinnvoll zu Readability/Defuddle Options)
}

export interface Article {
  title: string;
  byline?: string;
  dir?: string;
  lang?: string;
  content: string;         // HTML
  textContent?: string;    // optional, falls Defuddle liefert
  excerpt?: string;
  siteName?: string;
  length: number;
  // plus: Defuddle-spezifische Felder (z. B. markdown, cleanHtml, ...)
}

export function extractReadability(html: string, opts?: ExtractOptions): Article | null;
export function extractDefuddle(html: string, opts?: ExtractOptions): Article | null;
export function isProbablyReaderable(html: string, opts?: { minContentLength?: number }): boolean;
```

---

## 3) Repo-/Workspace-Layout (pnpm + Cargo Workspace)

> Ziel: „Rust als Engine“ + „TS als UX“ + striktes Tooling.

```
repo/
  crates/
    dom-shim/                 # DOM-Abstraktion (Element/Node/Document + Query/Traversal)
    readability-core/         # 1:1 Port von mozilla/readability
    defuddle-core/            # 1:1 Port von kepano/defuddle (Delta/Alternative Extractor)
    bindings-napi/            # napi-rs Bindings
  packages/
    node/                     # TS Wrapper Package (strict, eslint typed, prettier, vitest)
    fixtures-tools/           # optional: sync & normalize scripts (TS)
  upstream/
    readability/              # git subtree/submodule: mozilla/readability
    defuddle/                 # git subtree/submodule: kepano/defuddle
  fixtures/
    readability/              # gespiegelt aus upstream/readability/test/test-pages
    defuddle/                 # gespiegelt aus upstream/defuddle/tests (oder fixtures-Ordner)
  .github/workflows/
  Cargo.toml                  # workspace
  pnpm-workspace.yaml
```

### Upstream-Einbindung: Subtree vs Submodule
- **Submodule**: sauberer Upstream-Link, aber UX manchmal nervig.
- **Subtree**: einfacher für Konsumenten/CI, dafür Pull/Sync etwas manueller.

Empfehlung: **Subtree** für `upstream/*` (und zusätzlich ein `UPSTREAM.md` mit Commit-SHAs).  

---

## 4) Entwicklungsplan (phasenweise, mit klaren Exit-Kriterien)

### Phase 0 — Upstream „as-is“ lauffähig machen (JS)
**Ziele**
- Upstream-Tests lokal ausführbar machen (mindestens Readability).
- Sync-Mechanik etablieren (Subtree/Submodule + Scripts).

**Tasks**
- `upstream/readability` als subtree einziehen.
- `fixtures/readability` automatisiert aus `upstream/readability/test/test-pages` spiegeln.
- Minimaler CI-Job: `pnpm -C upstream/readability test` (sofern vorgesehen) oder zumindest „fixtures vorhanden & vollständig“.

**Exit-Kriterien**
- Fixture-Suite ist im Repo, CI findet sie reproduzierbar.

---

### Phase 1 — DOM-Shim in Rust (portierfähige Basis)
**Ziele**
- Eine DOM-Repräsentation, die die wichtigsten Readability/Defuddle-Operationen unterstützt.
- Fokus auf: Traversal, Tag-/Attr-Manipulation, Text/HTML Serialisierung, Query-Subset.

**Entscheidungen**
- Parser/DOM: z. B. `html5ever` + DOM-Wrapper (oder `kuchiki`).
- HTML-Serializer: deterministisch (wichtig für Golden Tests!).

**Exit-Kriterien**
- Aus `source.html` wird ein `Document` erzeugt und wieder deterministisch serialisiert.
- Grund-Query/Traversal-Tests bestehen.

---

### Phase 2 — Readability 1:1 Port (Kern)
**Ziele**
- Transliteration von `Readability.js` (und ggf. Readerable) nach Rust.
- Möglichst gleiche Datenstrukturen / Funktionsgrenzen / Heuristiken.

**Praktische Port-Mechanik**
- Datei-/Funktionsmapping: `Readability.js` → `readability_core/src/readability.rs` (oder modulare Splits, aber stabil).
- „Lineage“-Kommentare: Referenz auf Upstream-Datei + Commit + (optional) Zeilenbereiche.

**Exit-Kriterien**
- Erste Fixtures laufen: `001` + weitere „simple cases“ grün.

---

### Phase 3 — Readability Test-Port (Upstream-Fixtures + Oracle)
**Ziele**
- Upstream-Fixtures werden in Rust als Golden Tests ausgeführt.
- Optional: Oracle-Mode (JS-Referenzlauf) für zusätzliche Sicherheit.

**Testarten**
1. **Golden Fixture Tests (Rust):**
   - Input: `fixtures/readability/**/source.html`
   - Expected: `expected.html` (HTML) + erwartete Metadaten (falls vorhanden)
2. **Normalization Layer:**
   - Whitespace/Attribute-Normalisierung, damit Tests nicht wegen Serialisierungsdetails flaken.
3. **Oracle-Vergleich (optional, CI nightly):**
   - run JS Readability auf `source.html` → compare mit Rust output (diff tolerant konfigurierbar)

**Exit-Kriterien**
- Ein signifikanter Subset (z. B. 20–30 Cases) stabil grün.
- Danach sukzessive Abdeckung erhöhen.

---

### Phase 4 — Node Bindings + TS Quality Gate
**Ziele**
- N-API Binding mit sauberer TS-API.
- TS strict + typed-eslint strict + prettier + vitest.

**Tasks**
- `bindings-napi`: exports `extractReadability`, `isProbablyReaderable`, etc.
- `packages/node`: TS wrapper, Typen, Vitest Tests gegen Fixtures.

**Exit-Kriterien**
- `pnpm test` läuft:  
  - `cargo test` (Rust)  
  - `vitest` (Node)  
  - `eslint` (typed strict)  
  - `prettier --check`

---

### Phase 5 — Defuddle Delta-Analyse + 1:1 Port
**Ziele**
- Defuddle-Algorithmus 1:1 portieren, aber **auf Basis eines stabilen DOM-Shims**.
- Defuddle-Tests/Fixtures übernehmen und als zweite Golden-Suite integrieren.

**Vorgehen**
1. **Diff-Analyse**: Welche Schritte/Heuristiken unterscheiden sich von Readability?
2. **Port**: Defuddle-Core als eigenes Modul oder als „Strategy“ im gemeinsamen Extractor.
3. **Tests**: Defuddle `tests/` spiegeln und in Rust+Node ausführen.

**Exit-Kriterien**
- Defuddle-spezifische Tests grün.
- Keine Regression der Readability Suite.

---

## 5) Tooling-Vorgaben (deine Ziele) – konkret umgesetzt

### TypeScript Strict
- `tsconfig.json`: `"strict": true`, `"noUncheckedIndexedAccess": true`, `"exactOptionalPropertyTypes": true`, etc.

### ESLint Strict Typed
- `@typescript-eslint` mit type-aware rules (`parserOptions.project`)
- „strictest“ Set + gezielte Ausnahmen nur mit Kommentar/Begründung.

### Prettier Formatted
- Prettier als alleiniger Formatter; ESLint ohne stylistic conflicts.

### Vitest tested
- Node-Wrapper-Tests:
  - `fixtures/readability/*` & später `fixtures/defuddle/*`
  - Snapshot-Option nur, wenn Output stabil normalisiert ist.

### Rust Side
- `rustfmt` + `clippy -D warnings`
- `cargo test` (+ optional `nextest`)

---

## 6) Risiken & Gegenmaßnahmen (speziell bei 1:1 Ports)

1. **DOM-API Drift:** JS DOM ist sehr reich; Rust-DOM muss nur das abbilden, was Readability/Defuddle tatsächlich nutzt.  
   → DOM-Shim als „contract“, plus targeted unit tests.

2. **HTML-Serialisierung (golden tests):** Unterschiedliche Serializer erzeugen unterschiedliche Attribute/Whitespace.  
   → Normalize-Funktion + deterministischer Serializer.

3. **Performance vs Genauigkeit:** 1:1 kann weniger „idiomatisch“ sein.  
   → Erst Korrektheit, dann (später) optimieren – mit Perf-Benchmarks.

---

## 7) „Definition of Done“ (für ein erstes Release)

- ✅ Readability: signifikante Fixture-Abdeckung (z. B. 50+ Cases) grün  
- ✅ Node Package: TS strict, eslint typed strict, prettier, vitest grün  
- ✅ CI: lint + tests + build native bindings für gängige Targets  
- ✅ Lizenz: Upstream LICENSE/NOTICE sauber enthalten, Attribution dokumentiert  
- ✅ Dokumentation: API + „How to add testcases“ (inkl. Upstream-generate Ansatz)

---

## 8) Nächste konkrete Schritte (sehr pragmatisch)

1. Repo scaffolding (pnpm workspace + cargo workspace).  
2. `upstream/readability` per subtree einziehen + `fixtures/readability` spiegeln.  
3. DOM-Shim bauen (parse/serialize/traverse/query subset).  
4. Readability portieren: start mit minimalem parse flow und Case `001`.  
5. Golden test harness: `source.html` → `content` gegen `expected.html`.  
6. Node bindings (napi-rs) + Vitest Tests auf denselben Fixtures.  
7. Erst dann: Defuddle diff & Port.

