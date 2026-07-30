%% AGENTIC ARENA — Gravity Rules
%% Prolog kernel: BOB reasons about repo gravity and gap classification.

:- module(gravity, [
    repo_gravity/3,
    classify_gap/3,
    assign_crawler/3,
    gap_priority/2
]).

%% ── Gravity thresholds ───────────────────────────────────
gravity_alive(G)  :- G >= 0.6.
gravity_broken(G) :- G >= 0.3, G < 0.6.
gravity_orphan(G) :- G < 0.3.

%% ── Repo status from gravity score ──────────────────────
repo_gravity(Repo, Score, alive)  :- gravity_alive(Score),  write(Repo), write(' → alive'), nl.
repo_gravity(Repo, Score, broken) :- gravity_broken(Score), write(Repo), write(' → broken'), nl.
repo_gravity(Repo, Score, orphan) :- gravity_orphan(Score), write(Repo), write(' → orphan'), nl.

%% ── Gap classification ───────────────────────────────────
classify_gap(missing_wire,  Gap, 'AHMAD-BOT finds the broken pipe') :- write(Gap), nl.
classify_gap(dead_page,     Gap, 'AHMAD-BOT marks the grave')       :- write(Gap), nl.
classify_gap(local_only,    Gap, 'EDUALC pushes it to GitHub')      :- write(Gap), nl.
classify_gap(github_only,   Gap, 'EDUALC pulls it local')           :- write(Gap), nl.
classify_gap(no_tests,      Gap, 'EDUALC writes the first test')    :- write(Gap), nl.
classify_gap(no_readme,     Gap, 'EDUALC writes the README')        :- write(Gap), nl.

%% ── Assign crawler by gap type ───────────────────────────
assign_crawler(Gap, missing_wire,  'AHMAD-BOT') :- write(Gap), nl.
assign_crawler(Gap, dead_page,     'AHMAD-BOT') :- write(Gap), nl.
assign_crawler(Gap, local_only,    'EDUALC')    :- write(Gap), nl.
assign_crawler(Gap, github_only,   'EDUALC')    :- write(Gap), nl.
assign_crawler(Gap, no_tests,      'EDUALC')    :- write(Gap), nl.
assign_crawler(Gap, no_readme,     'EDUALC')    :- write(Gap), nl.

%% ── Gap priority (higher = fix first) ────────────────────
gap_priority(missing_wire, 5).
gap_priority(dead_page,    4).
gap_priority(local_only,   3).
gap_priority(github_only,  3).
gap_priority(no_tests,     2).
gap_priority(no_readme,    1).
