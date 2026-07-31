%% system_invariants.pl — bifrost global chain invariants
%%
%% Loaded by bifrost-policy engine at startup.
%% Every predicate here is a MUST — violation triggers chain halt.
%%
%% Key invariant numbering matches the bifrost specification.

:- module(system_invariants, [
    valid_transfer/4,
    valid_jit_compile/3,
    chain_valid/1,
    worm_sealed/1       % provided by Rust FFI bridge
]).

%% INV-1: No cap transfer without valid policy CID sealed in WORM_FS.
%%
%% valid_transfer(+From, +To, +CapCID, +PolicyCID)
valid_transfer(From, To, Cap, PolicyCID) :-
    worm_sealed(PolicyCID),        %% policy must be immutable
    cap_owner(Cap, From),          %% caller must own the cap
    policy_allows(PolicyCID, transfer, From, To),
    \+ cap_revoked(Cap).           %% revocation check

%% INV-2: Every JIT compile must reference a SoulIR CID that exists in WORM_FS.
%%        The output WASM CID must be NEW (not yet sealed).
%%
%% valid_jit_compile(+SoulIR_CID, +Wasm_CID, +OptLevel)
valid_jit_compile(SoulIR_CID, Wasm_CID, _OptLevel) :-
    worm_sealed(SoulIR_CID),
    \+ worm_sealed(Wasm_CID).

%% INV-3: Chain continuity — every event links back to genesis.
%%
%% chain_valid(+HeadCID)
chain_valid(genesis).
chain_valid(Head) :-
    event(Head, Prev, _, _),
    chain_valid(Prev).

%% INV-4: Height monotonicity.
%%
%% height_monotone(+EventCID)
height_monotone(CID) :-
    event(CID, Prev, Height, _),
    (   Prev = genesis
    ->  Height =:= 0
    ;   event(Prev, _, PrevHeight, _),
        Height =:= PrevHeight + 1
    ).

%% INV-5: Signature freshness — pubkey must not be in the revocation list.
%%
%% pubkey_valid(+PubKeyHex)
pubkey_valid(PubKey) :-
    \+ revoked_pubkey(PubKey).
