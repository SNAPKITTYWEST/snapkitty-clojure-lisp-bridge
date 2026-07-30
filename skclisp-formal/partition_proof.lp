% SKC-LISP-WORLD: Self-Modifying ASP for Euler Pentagonal Recurrence
% Computes partition numbers and emits Lean 4 proof trace

% ============================================================================
% BASE FACTS: Partition Number Definition
% ============================================================================

% p(0) = 1 (base case)
partition(0, 1).

% Pentagonal numbers: g_k = k(3k-1)/2 for k ≠ 0
% Generate for positive k: k=1,2,3,...
gen_pentagonal(K, G) :- K > 0, K <= 100, G = K * (3 * K - 1) / 2.

% Generate for negative k: k=-1,-2,-3,...
gen_pentagonal(K, G) :- K < 0, K >= -100, KPos = -K, G = KPos * (3 * KPos + 1) / 2.

% Sign of term: (-1)^(k-1)
% For k > 0: sign alternates 1, -1, 1, -1, ...
sign_term(K, Sign) :- K > 0, K mod 2 = 1, Sign = 1.
sign_term(K, Sign) :- K > 0, K mod 2 = 0, Sign = -1.

% For k < 0: same but offset
sign_term(K, Sign) :- K < 0, KAbs = -K, KAbs mod 2 = 0, Sign = 1.
sign_term(K, Sign) :- K < 0, KAbs = -K, KAbs mod 2 = 1, Sign = -1.

% ============================================================================
% SELF-MODIFYING RULES: Generate Recurrence for Each N
% ============================================================================

#program step(N).

% For each N ≥ 1, generate the recurrence rule
% partition(N, V) = sum { sign * partition(N - g_k) : g_k is pentagonal }

% Rule: Collect all valid pentagonal contributions
recurrence_term(N, K, G, S, V) :-
    N > 0,
    N <= 100,
    gen_pentagonal(K, G),
    G <= N,
    sign_term(K, S),
    N_minus_G = N - G,
    partition(N_minus_G, V).

% Compute partition(N) as sum of signed terms
partition_sum(N, Sum) :-
    N > 0,
    N <= 100,
    Sum = #sum { S * V : recurrence_term(N, K, G, S, V) }.

% Constraint: partition(N) must match computed sum
partition(N, Sum) :-
    N > 0,
    partition_sum(N, Sum).

% ============================================================================
% DERIVATION TRACE: Record computation steps for Lean proof
% ============================================================================

% Trace base case
derivation_step(base_case, "p(0) = 1", 0, 1).

% Trace each recurrence computation
derivation_step(recurrence, "p(@n) computed from pentagonal terms", N, V) :-
    partition(N, V),
    N > 0.

% Trace individual term contributions
term_contribution(N, K, G, S, V) :-
    recurrence_term(N, K, G, S, V).

% ============================================================================
% VERIFICATION: Check OEIS A000041 (partition numbers)
% ============================================================================

% Known partition numbers for verification
known_partition(0, 1).
known_partition(1, 1).
known_partition(2, 2).
known_partition(3, 3).
known_partition(4, 5).
known_partition(5, 7).
known_partition(6, 11).
known_partition(7, 15).
known_partition(8, 22).
known_partition(9, 30).
known_partition(10, 42).
known_partition(15, 176).
known_partition(20, 627).

% Verify computed values
verify_partition(N) :- partition(N, V), known_partition(N, V).
verify_partition(N) :- partition(N, V), not known_partition(N, _).

% ============================================================================
% OUTPUT: Generate for Lean 4
% ============================================================================

#show partition/2.
#show term_contribution/5.
#show derivation_step/4.
#show verify_partition/1.
