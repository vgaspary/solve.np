import { maybe_vote } from './p_vs_np_utils';
import { check_solution } from './problem_utils';
import { maybe_delete, new_attempt, cancel_new_attempt, create_solution, submit_solution, choose_solution } from './solution_utils';

declare global {
  interface Window {
    solve_np: any;
  }
}

window.solve_np = {
  maybe_vote,
  check_solution,
  maybe_delete,
  new_attempt,
  cancel_new_attempt,
  create_solution,
  submit_solution,
  choose_solution,
};
