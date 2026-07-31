%% SKC-LISP: Layered Truth Maintenance System (LTMS)
%% Knowledge layer: Prolog symbolic reasoning engine
%% Author: Ahmad Parr (ahmedparr93@gmail.com)
%% Integration: Snapkitty LISP Bridge knowledge engine

:- dynamic fact/5.        % fact(Value, Source, Timestamp, Confidence, Priority)
:- dynamic rule/4.        % rule(Head, Body, Module, Priority)
:- dynamic concept/2.     % concept(Name, Senses)
:- dynamic sense/4.       % sense(Concept, Gloss, ContextPred, Confidence)
:- dynamic belief_base/1. % belief_base(Timestamp) for history
:- dynamic module_stats/3. % module_stats(Module, RuleCount, LastUpdated)

%% ============================================================================
%% 1. CONFLICT RESOLUTION
%% ============================================================================
%% When multiple facts claim the same value, pick the winner by:
%% Priority > Confidence > Source recency

resolve_conflict(Value, Winner) :-
    findall(f(Value,S,T,C,P), fact(Value,S,T,C,P), Candidates),
    (Candidates = []
      -> fail
      ;  predsort(compare_facts, Candidates, Sorted),
         Sorted = [Winner|_]).

%% Comparison: higher priority wins, then higher confidence
compare_facts(Order, f(_,_,_,C1,P1), f(_,_,_,C2,P2)) :-
    ( P1 > P2 -> Order = (<)
    ; P1 < P2 -> Order = (>)
    ; C1 > C2 -> Order = (<)
    ; C1 < C2 -> Order = (>)
    ; Order = (=) ).

%% Get the winning value
get_belief(Value, WinningFact) :-
    resolve_conflict(Value, WinningFact).

%% ============================================================================
%% 2. OUTDATED DETECTION
%% ============================================================================
%% Exponential decay: confidence decays over time
%% Decay function: Conf(t) = Conf(0) * exp(-0.0001 * age_in_ms)

is_outdated(fact(_,_,Created,Conf,_), Now) :-
    Decay is Conf * exp(-0.0001 * (Now - Created)),
    Decay < 0.15.

prune_outdated_facts(Now) :-
    findall(f(V,S,T,C,P), fact(V,S,T,C,P), AllFacts),
    forall(member(F, AllFacts),
           (is_outdated(F, Now) -> retract(F) ; true)).

%% Half-life: time for confidence to drop to 50%
compute_half_life(HalfLife) :-
    HalfLife is log(2.0) / 0.0001.  % ~6931 milliseconds = 6.9 seconds

%% ============================================================================
%% 3. AMBIGUOUS CONCEPTS
%% ============================================================================
%% Concepts have multiple senses, each with a context predicate

register_concept(Name, SenseGlosses) :-
    findall(sense(Name, Gloss, _, 1.0), member(Gloss, SenseGlosses), Senses),
    assertz(concept(Name, Senses)).

disambiguate(Concept, Context, Sense) :-
    concept(Concept, Senses),
    member(sense(Concept, Gloss, Pred, Conf), Senses),
    (var(Pred) ; call(Pred, Context)),
    Sense = sense(Concept, Gloss, Pred, Conf).

%% Pick best sense by confidence (or first if all equal)
best_sense(Concept, Context, BestSense) :-
    findall(Sense, disambiguate(Concept, Context, Sense), Candidates),
    (Candidates = []
      -> fail
      ;  sort(Candidates, [BestSense|_])).

%% ============================================================================
%% 4. MAINTAINABILITY GUARD
%% ============================================================================
%% Hard limit: no module can have > 80 rules
%% Prevents knowledge explosion and maintains readable rule sets

assert_rule(Head, Body, Module, Priority) :-
    findall(_, rule(_,_,Module,_), Rules),
    length(Rules, N),
    (N >= 80
      -> throw(error(module_overflow,
                     context(assert_rule/4,
                             "Module " + Module + " has reached rule limit")))
      ;  assertz(rule(Head, Body, Module, Priority)),
         update_module_stats(Module)).

update_module_stats(Module) :-
    (retract(module_stats(Module, _, _)) ; true),
    findall(_, rule(_,_,Module,_), Rules),
    length(Rules, Count),
    get_time(Now),
    assertz(module_stats(Module, Count, Now)).

%% Get module health (rules / max_rules)
module_complexity(Module, Percentage) :-
    module_stats(Module, Count, _),
    Percentage is (Count / 80.0) * 100.

%% Suggest refactoring when > 70% full
suggest_refactor(Module) :-
    module_complexity(Module, P),
    P > 70.0.

%% ============================================================================
%% 5. HYBRID / NON-RULE KNOWLEDGE
%% ============================================================================
%% Fallback to embeddings when symbolic reasoning fails

hybrid_prove(Goal, Result, Confidence) :-
    ( prove_symbolic(Goal, Result)
    -> Confidence = 1.0
    ;  embedding_retrieve(Goal, Result, Confidence)
    ).

prove_symbolic(Goal, Result) :-
    clause(Goal, Body),
    call(Body),
    Result = Goal.

%% Stub for embedding retrieval (connects to vector DB)
embedding_retrieve(Goal, Result, Confidence) :-
    % In production: query Qdrant/WORM ledger for similar embeddings
    % For now: fail gracefully
    fail.

%% ============================================================================
%% ASSERTION & QUERY API
%% ============================================================================

%% Assert a new fact
assert_fact(Value, Source, Confidence, Priority) :-
    get_time(Now),
    Timestamp is floor(Now * 1000),  % milliseconds
    assertz(fact(Value, Source, Timestamp, Confidence, Priority)).

%% Query belief about a value (returns winning fact + confidence)
query(Value, Confidence) :-
    resolve_conflict(Value, f(Value, _, _, Confidence, _)).

%% All known senses of a concept
all_senses(Concept, AllSenses) :-
    concept(Concept, AllSenses).

%% Concept subsumption: is Specific a kind of General?
is_a(Specific, General) :-
    fact(is_a(Specific, General), _, _, _, _).

%% ============================================================================
%% DEBUGGING & INSPECTION
%% ============================================================================

%% List all facts
list_facts :-
    findall(f(V,S,T,C,P), fact(V,S,T,C,P), Facts),
    forall(member(F, Facts), writeln(F)).

%% List all rules by module
list_rules_in_module(Module) :-
    findall(rule(H,B,M,Pr), rule(H,B,Module,Pr), Rules),
    forall(member(R, Rules), writeln(R)).

%% List all concepts
list_concepts :-
    findall(C, concept(C, _), Concepts),
    forall(member(C, Concepts), writeln(C)).

%% Module report
module_report :-
    findall(m(M,Count,Updated), module_stats(M,Count,Updated), Stats),
    forall(member(m(Mod,Count,_), Stats),
           format('~w: ~w/80 rules~n', [Mod, Count])).

%% ============================================================================
%% TESTS
%% ============================================================================

:- initialization(run_ltms_tests).

run_ltms_tests :-
    writeln('=== LTMS Tests ==='),

    % Test 1: Conflict resolution
    writeln('Test 1: Conflict resolution'),
    assert_fact(color(car), source_a, 0.8, 10),
    assert_fact(color(car), source_b, 0.9, 5),
    resolve_conflict(color(car), Winner),
    writeln(Winner), nl,

    % Test 2: Outdated detection
    writeln('Test 2: Outdated detection'),
    get_time(Now),
    OldTime is (Now * 1000) - 10000,
    (is_outdated(fact(old_value, old_source, OldTime, 0.5, 1), Now * 1000)
      -> writeln('✓ Old fact marked as outdated')
      ;  writeln('✗ Old fact not detected')), nl,

    % Test 3: Module limit
    writeln('Test 3: Module complexity guard'),
    (module_complexity(main, P), format('Main module: ~1f% full~n', [P])
      ;  writeln('Main module not yet used')), nl,

    writeln('=== LTMS Tests Complete ===').
