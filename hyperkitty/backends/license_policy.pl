% license_policy.pl — Tri-licensing policy engine with Prolog cartography
% BSL + AGPL-3.0 + MPL/dual-license compatibility reasoning

:- module(license_policy, [
    check_compatibility/2,
    can_link/2,
    requires_disclosure/1,
    commercial_allowed/1,
    classify_license/2,
    filter_dependencies/3,
    enforce_policy/3
]).

% ============================================================================
% LICENSE TAXONOMY
% ============================================================================

% Strong copyleft (network effect)
strong_copyleft(agpl3).
strong_copyleft('AGPL-3.0').
strong_copyleft('AGPL-3.0-only').

% Weak copyleft (file-level)
weak_copyleft(mpl2).
weak_copyleft('MPL-2.0').
weak_copyleft(lgpl3).
weak_copyleft('LGPL-3.0').

% Business Source License (source-available, restricted commercial use)
source_available_restricted(bsl).
source_available_restricted('BSL-1.1').
source_available_restricted('Business Source License 1.1').

% Permissive
permissive(mit).
permissive('MIT').
permissive(apache2).
permissive('Apache-2.0').
permissive(bsd3).
permissive('BSD-3-Clause').

% Proprietary/Commercial
commercial(proprietary).
commercial(commercial).

% ============================================================================
% COMPATIBILITY RULES
% ============================================================================

% Strong copyleft can absorb anything (but forces AGPL on the whole)
can_link(agpl3, X) :-
    (strong_copyleft(X) ; weak_copyleft(X) ; permissive(X)).

% BSL can link with permissive, but not copyleft
can_link(bsl, X) :-
    (source_available_restricted(X) ; permissive(X)).

% MPL can link with permissive and other MPL
can_link(mpl2, X) :-
    (weak_copyleft(X) ; permissive(X)).

% Permissive can link with anything
can_link(L, _) :-
    permissive(L).

% Commercial licenses need explicit compatibility
can_link(commercial, X) :-
    (commercial(X) ; permissive(X)).

% Symmetric compatibility (commutative)
compatible(A, B) :- can_link(A, B), can_link(B, A).

% One-way compatibility (A can depend on B, but not vice versa)
one_way_compatible(A, B) :- can_link(A, B), \+ can_link(B, A).

% ============================================================================
% DISCLOSURE REQUIREMENTS
% ============================================================================

% These licenses require source disclosure when distributed
requires_disclosure(L) :- strong_copyleft(L).
requires_disclosure(L) :- weak_copyleft(L).
requires_disclosure(L) :- source_available_restricted(L).

% Network copyleft: triggers even over network (SaaS)
requires_network_disclosure(L) :- strong_copyleft(L).

% File-level copyleft: only modified files need disclosure
requires_file_disclosure(L) :- weak_copyleft(L).

% ============================================================================
% COMMERCIAL USE RULES
% ============================================================================

% Can use in commercial product without restriction
commercial_allowed(L) :- permissive(L).
commercial_allowed(L) :- weak_copyleft(L).  % MPL allows commercial if file-level separation maintained

% Restricted commercial use (BSL: no managed service offerings at scale)
commercial_restricted(L) :- source_available_restricted(L).

% Requires commercial license to bypass copyleft
requires_commercial_license(L) :- strong_copyleft(L).

% ============================================================================
% LICENSE CLASSIFICATION
% ============================================================================

classify_license(License, Category) :-
    (strong_copyleft(License) -> Category = strong_copyleft ;
     weak_copyleft(License) -> Category = weak_copyleft ;
     source_available_restricted(License) -> Category = source_available ;
     permissive(License) -> Category = permissive ;
     commercial(License) -> Category = commercial ;
     Category = unknown).

% ============================================================================
% DEPENDENCY FILTERING
% ============================================================================

% filter_dependencies(+ProjectLicense, +DepList, -ValidDeps)
% Given a project license and list of dependencies, return only compatible ones
filter_dependencies(_, [], []).
filter_dependencies(ProjectLicense, [dep(Name, DepLicense)|Rest], [dep(Name, DepLicense)|ValidRest]) :-
    can_link(ProjectLicense, DepLicense),
    !,
    filter_dependencies(ProjectLicense, Rest, ValidRest).
filter_dependencies(ProjectLicense, [_|Rest], ValidRest) :-
    filter_dependencies(ProjectLicense, Rest, ValidRest).

% ============================================================================
% POLICY ENFORCEMENT
% ============================================================================

% enforce_policy(+ProjectLicense, +Dependencies, -Result)
% Check if all dependencies are compatible with project license
enforce_policy(ProjectLicense, Dependencies, Result) :-
    check_all_compatible(ProjectLicense, Dependencies, [], Violations),
    (Violations = [] ->
        Result = pass(compatible_all) ;
        Result = fail(violations(Violations))
    ).

check_all_compatible(_, [], Violations, Violations).
check_all_compatible(ProjectLicense, [dep(Name, DepLicense)|Rest], Acc, Violations) :-
    (can_link(ProjectLicense, DepLicense) ->
        check_all_compatible(ProjectLicense, Rest, Acc, Violations) ;
        check_all_compatible(ProjectLicense, Rest, [incompatible(Name, DepLicense)|Acc], Violations)
    ).

% ============================================================================
% TRI-LICENSE STRATEGY
% ============================================================================

% SnapKitty tri-license: BSL + AGPL-3.0 + MPL/Commercial dual-license
snapkitty_primary(bsl).
snapkitty_copyleft(agpl3).
snapkitty_weak(mpl2).
snapkitty_commercial(commercial).

% When can someone use code under each layer?
can_use_under(bsl, Condition) :-
    Condition = 'Source-available, no managed services at scale, converts to AGPL after transition period'.

can_use_under(agpl3, Condition) :-
    Condition = 'Full copyleft, network distribution triggers disclosure, all modifications must be AGPL'.

can_use_under(mpl2, Condition) :-
    Condition = 'Weak copyleft, file-level, can combine with proprietary code, modified files must stay MPL'.

can_use_under(commercial, Condition) :-
    Condition = 'Commercial license bypasses copyleft restrictions, negotiated terms'.

% Which license applies to a given use case?
select_license_for_use(managed_service, agpl3) :-
    % If deploying as managed service → AGPL triggers network copyleft
    !.
select_license_for_use(saas_wrapper, agpl3) :-
    % If wrapping as SaaS → AGPL forces disclosure
    !.
select_license_for_use(enterprise_scale, bsl) :-
    % If enterprise scale without managed service → BSL applies
    !.
select_license_for_use(file_modification, mpl2) :-
    % If only modifying specific files → MPL file-level copyleft
    !.
select_license_for_use(commercial_bypass, commercial) :-
    % If wants to bypass copyleft → commercial license required
    !.
select_license_for_use(open_source_contribution, agpl3) :-
    % Default open source path → AGPL
    !.

% ============================================================================
% COMPATIBILITY MATRIX QUERY
% ============================================================================

% Generate full compatibility matrix
generate_matrix(Matrix) :-
    findall(compatible(A, B),
            (license_type(A), license_type(B), compatible(A, B)),
            Matrix).

license_type(agpl3).
license_type(bsl).
license_type(mpl2).
license_type(mit).
license_type(apache2).
license_type(commercial).

% ============================================================================
% EXAMPLES / TESTS
% ============================================================================

% Example: Can AGPL project depend on MIT library? YES
% ?- can_link(agpl3, mit).

% Example: Can BSL project depend on AGPL library? NO
% ?- can_link(bsl, agpl3).

% Example: Filter dependencies for BSL project
% ?- filter_dependencies(bsl, [dep(foo, mit), dep(bar, agpl3), dep(baz, apache2)], Valid).
% Valid = [dep(foo, mit), dep(baz, apache2)].

% Example: Enforce policy
% ?- enforce_policy(agpl3, [dep(lib1, mit), dep(lib2, apache2)], Result).
% Result = pass(compatible_all).

% Example: Select license for SaaS use
% ?- select_license_for_use(saas_wrapper, License).
% License = agpl3.

% ============================================================================
% CLI INTERFACE
% ============================================================================

:- initialization(main, main).

main(Argv) :-
    (Argv = [check, ProjectLicense, DepFile] ->
        check_dependencies_from_file(ProjectLicense, DepFile) ;
     Argv = [matrix] ->
        print_compatibility_matrix ;
     Argv = [select, UseCase] ->
        (atom_string(UseCaseAtom, UseCase),
         select_license_for_use(UseCaseAtom, License),
         format('License for ~w: ~w~n', [UseCase, License])) ;
        print_usage
    ),
    halt(0).

main(_) :-
    halt(1).

print_usage :-
    writeln('Usage:'),
    writeln('  swipl -q -t halt -f license_policy.pl -- check <license> <deps.json>'),
    writeln('  swipl -q -t halt -f license_policy.pl -- matrix'),
    writeln('  swipl -q -t halt -f license_policy.pl -- select <use_case>'),
    writeln(''),
    writeln('Examples:'),
    writeln('  swipl ... -- check agpl3 deps.json'),
    writeln('  swipl ... -- matrix'),
    writeln('  swipl ... -- select saas_wrapper').

print_compatibility_matrix :-
    writeln('License Compatibility Matrix:'),
    writeln('============================'),
    forall(
        (license_type(A), license_type(B)),
        (format('~w -> ~w: ', [A, B]),
         (can_link(A, B) -> writeln('YES') ; writeln('NO')))
    ).

check_dependencies_from_file(ProjectLicense, DepFile) :-
    atom_string(ProjectLicenseAtom, ProjectLicense),
    % Load dependencies from JSON file
    % Stub for now — would use library(http/json) in real implementation
    writeln('Dependency checking not yet implemented'),
    fail.
