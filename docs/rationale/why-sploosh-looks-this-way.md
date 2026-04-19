# Why Sploosh Looks the Way It Does

*Mechanistic grounding for the design principles in [`docs/spec-plans/LANGUAGE_SPEC.md`](../spec-plans/LANGUAGE_SPEC.md) §1.*

This document is not normative. It records the research findings that make Sploosh's design principles more than taste — the mechanistic reasons those principles should produce measurable gains in LLM generation accuracy.

## Three independent findings

Three strands of recent interpretability and representation research converge on a single picture of what transformers actually do when they read and write text.

### 1. LLMs reason in a format-agnostic middle

Cross-language probes on large modern transformers (e.g. a 64-layer Qwen 3.5 27B) reveal three distinct zones of computation:

- **Early layers (~first 5):** encoding. The model normalizes raw input from different scripts and surface forms into an internal representation.
- **Middle layers (~10–45):** a format-agnostic space. Same-content/different-language pairs measure more similar (~0.902 on a 0–1 scale) than same-language/different-content pairs (~0.891). Meaning dominates; surface language becomes nearly invisible.
- **Late layers (~last 15):** decoding. The model recommits to a specific output language.

A related experiment: **duplicating a complete middle "circuit"** — a contiguous functional block of layers — improves multi-step reasoning without adding knowledge. The model isn't learning new facts; it's getting another pass through the same reasoning workshop. Duplicating a single isolated layer, or crossing a circuit boundary by one layer, destroys the gain entirely. Reasoning lives in groups, not in individual layers.

### 2. The Platonic Representation Hypothesis

Huh, Cheung, Wang & Isola (MIT, 2024) measured 78 vision models and a suite of language models and found that internal representations **converge** as models get bigger and better — and not just within a modality. Vision and language models trained on completely different data start measuring similarity between data points in increasingly similar ways. The hypothesis is not uncontested (absolute alignment scores remain modest, and critics note that benchmark datasets may share more information than intended), but the *direction* is consistent across architectures, labs, and modalities.

### 3. Looped models outperform at fixed capacity

A transformer that loops the same block of middle layers multiple times during training matches the knowledge capacity of a non-looped model of the same parameter count but outperforms it on reasoning. Controlled experiments separate the variables: knowledge capacity identical, reasoning better. Small looped models compete with dense models 2–5× their size. Two additional findings reinforce the picture:

- Reasoning traces computed inside the loop (in the format-agnostic middle) are more faithful to the final output than standard chain-of-thought text — which is often post-hoc rationalization.
- Harmfulness on safety benchmarks decreases as recurrent depth increases, even beyond the depth the model was trained at.

All three findings are independent, and they agree: the middle layers of a transformer operate in a representational space that is not any human language, not specific to any single model, and becomes more universal as capability scales.

## What this means for Sploosh

### Sploosh is a decode-optimized surface language

The thinking happens in the middle, in a space with no language. Sploosh cannot change that, and doesn't need to. What Sploosh *can* change is the cost of the final translation from meaning to tokens. Every ambiguity — "is it `format!` or an f-string?", "does this coerce silently?", "which of three comment styles applies?" — forces disambiguation in the last few layers, exactly where the model has the fewest remaining passes to correct course.

This rationalizes several existing principles more sharply than "it's easier to learn":

- **§1.1 One way to do everything** — minimizes the token-level decision space. Lower decode entropy means fewer off-by-one token errors at the exact layers where those errors are hardest to recover from.
- **§1.2 Familiar vocabulary only** — the "top-N trained languages" heuristic is a bet on the dense, converged region of the Platonic representation. That region should remain stable across model generations, which is the kind of bet a language specification wants to make.
- **§1.3 Explicit over implicit** — removes hidden decisions that would otherwise have to be resolved silently in late layers with no structural guidance.
- **§1.4 Errors are values** — no hidden control flow means the model's structural thought projects cleanly into the AST. There is no exception-unwinding machinery for late layers to infer.
- **§1.5 Concurrency is structural** — actor boundaries and message passing are reasoned about structurally, matching how middle-layer representations appear to organize control flow.

### Reasoning scales with passes through the middle

Layer-duplication and looping say the same thing from different directions: reasoning quality is a function of how many middle-layer passes the model gets to spend on the problem. Surface bulk competes with reasoning for that budget — every layer spent parsing an unusual token is a layer not spent reasoning about the task.

This rationalizes:

- **§1.7 Spec fits in a prompt** — compactness is not only an ergonomic goal. The less budget a model spends encoding Sploosh's quirks (early layers) and decoding them back (late layers), the more of its middle-layer passes remain for the actual problem. A prompt-sized spec is, mechanistically, a reasoning-budget argument.
- **Zero tokenizer ambiguity (§2, §3)** — the same argument at the token level. The fewer middle passes it takes to even *identify* the token stream's structure, the more are left for semantics.

### Structure over narration

If chain-of-thought is often post-hoc rationalization while the real reasoning lives in the format-agnostic middle, then **code comments describing reasoning cannot be trusted to reflect actual reasoning** — whether written by a human or an LLM. Sploosh's existing choices (no block comments, `Result<T, E>` over exceptions, two-level visibility, actors over shared mutable state) all push meaning into the structure rather than into prose.

A candidate principle worth discussing — not proposed here as a change to §1, just flagged as a follow-up:

> **Structure over narration.** The model reasons in a space with no words. Code should encode meaning structurally; narration in comments is commentary, not evidence.

## What this does *not* argue

- **Not a claim of optimality.** Three research threads pointing the same direction as Sploosh's defaults is evidence of good taste, not proof of correctness. Specific thresholds — how compact is "prompt-sized", which exact tokens count as "familiar" — remain empirical calibration.
- **Not a claim that surface form is irrelevant.** The opposite. Early and late layers do meaningful work; "format-agnostic middle" means only that the *middle* ignores surface form. Because reasoning lives in the middle, surface form matters most at the edges — which is precisely where Sploosh intervenes.
- **Not a commitment to any specific architecture.** Sploosh does not depend on loop-LLMs, on any particular model family, or on the Platonic hypothesis being strongly true. It only requires that some version of these findings holds. The convergence of three independent threads suggests it does.
- **Not tool guidance.** These findings say nothing about editor integrations, build systems, or package management. Those stand on their own rationale.

## References

Primary citation is a vlog that summarizes all three threads and was the entry point for this document:

- *AI Doesn't Think in English* — <https://youtu.be/X5XKayn18ck>

Underlying research referenced by the vlog (consult primary sources before citing in spec text):

- Huh, Cheung, Wang & Isola, *The Platonic Representation Hypothesis* (MIT, 2024). Internal representations converge across models and modalities as scale and capability grow.
- Looped-transformer work co-authored by Y. Bengio and collaborators at ByteDance, training a recurrent-depth language model on ~7.7T tokens. Recurrent depth improves reasoning at fixed knowledge capacity; loop-internal states are more faithful than chain-of-thought text; harmfulness decreases with recurrent depth.
- Citizen-science replication on Qwen 3.5 27B measuring per-layer cross-language representation similarity and demonstrating targeted circuit duplication. Specific numbers in this document (0.902 / 0.891, ~17.72% reasoning gain at <10% added parameters) are quoted from the vlog; the underlying write-up should be linked here once verified.
