%% jit_compile.pl — Prolog policy for soulvm JIT compile events
%%
%% Enforced before a JitCompile event is accepted into the DAG.

:- module(jit_compile, [
    valid_jit_request/4,
    opt_level_permitted/2,
    gas_budget_valid/2
]).

%% valid_jit_request(+SoulCID, +SoulIR_CID, +OptLevel, +GasEstimate)
%%
%% A soul may compile if:
%%   1. The SoulIR source is sealed in WORM (immutable input).
%%   2. The requested opt_level is within the soul's trust budget.
%%   3. The gas estimate is non-zero and within the soul's allocation.
valid_jit_request(SoulCID, SoulIR_CID, OptLevel, GasEstimate) :-
    worm_sealed(SoulIR_CID),
    opt_level_permitted(SoulCID, OptLevel),
    gas_budget_valid(SoulCID, GasEstimate).

%% opt_level_permitted(+SoulCID, +Level)
%%
%% Level 0 = no optimisation (always allowed).
%% Level 1 = speed (requires trust ≥ 64).
%% Level 2 = speed_and_size (requires trust ≥ 128).
opt_level_permitted(_, 0) :- !.
opt_level_permitted(SoulCID, 1) :-
    trust_level(SoulCID, T), T >= 64, !.
opt_level_permitted(SoulCID, 2) :-
    trust_level(SoulCID, T), T >= 128.

%% gas_budget_valid(+SoulCID, +GasEstimate)
%%
%% Gas estimate must be > 0 and ≤ soul's gas allocation.
%% gas_allocation/2 is asserted by Rust bridge from CSpace.
gas_budget_valid(SoulCID, Gas) :-
    Gas > 0,
    gas_allocation(SoulCID, Alloc),
    Gas =< Alloc.

%% Default: all souls get 1_000_000 gas units.
gas_allocation(_, 1_000_000).

%% trust_level re-exported from cap_transfer module.
:- use_module(cap_transfer, [trust_level/2]).
