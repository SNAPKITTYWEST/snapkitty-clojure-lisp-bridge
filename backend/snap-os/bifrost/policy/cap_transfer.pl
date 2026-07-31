%% cap_transfer.pl — Prolog policy for silverback capability transfers
%%
%% Loaded by bifrost-policy. All rules here are ENFORCED before a
%% CapTransfer event is accepted into the DAG.

:- module(cap_transfer, [
    policy_allows/4,
    cap_owner/2,
    cap_revoked/1,
    trust_level/2
]).

%% policy_allows(+PolicyCID, +Action, +From, +To)
%%
%% Succeeds if the policy document at PolicyCID permits Action between From and To.
%% The actual policy document content is loaded from WORM_FS and asserted as facts
%% by the bifrost-policy Rust bridge before this rule is called.
policy_allows(PolicyCID, transfer, From, To) :-
    trust_level(From, FromTrust),
    trust_level(To, ToTrust),
    FromTrust >= 128,             %% sender needs elevated trust
    ToTrust   >= 0,               %% receiver has no minimum (can receive at any level)
    worm_sealed(PolicyCID),
    \+ transfer_blocked(From, To).

%% trust_level(+SoulCID, -Level)
%%
%% Silverback (root) = 255, council agents ≥ 128, user agents < 128.
%% Facts asserted by Rust bridge from the live CSpace.
trust_level(_, 128) :- !.  %% default: elevated trust (override per deployment)

%% cap_owner(+CapCID, +SoulCID)
%%
%% True when SoulCID holds a non-null cap with matching hash.
%% Facts asserted by Rust bridge from the current CSpace snapshot.
cap_owner(_, _) :- !.  %% stub — replaced by bridge

%% cap_revoked(+CapCID)
%%
%% Fails by default (no caps revoked until revocation is recorded in chain).
cap_revoked(_) :- fail.

%% transfer_blocked(+From, +To)
%%
%% Explicitly blocked transfers (e.g. Koko → Silverback is forbidden).
transfer_blocked(koko, silverback).

%% Delegate rights check — delegate bit must be set.
delegate_allowed(CapCID) :-
    cap_rights(CapCID, Rights),
    Rights /\ 0b01000 =:= 0b01000.

%% seal_allowed(CapCID) — seal bit must be set.
seal_allowed(CapCID) :-
    cap_rights(CapCID, Rights),
    Rights /\ 0b10000 =:= 0b10000.

%% Stub: cap_rights asserted by Rust bridge from CSpace.
cap_rights(_, 0b11111).  %% default: full rights
