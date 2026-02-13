import { maybe_vote } from './p_vs_np_utils';
import { check_solution } from './problem_utils';
import {
  edit_solution, edit_solution_title_desc, maybe_delete, new_attempt, cancel_new_attempt, create_solution, submit_solution, choose_solution, maybe_expand
} from './solution_utils';

declare global {
  interface Window {
    solve_np: any;
  }
}

window.solve_np = {
  edit_solution,
  edit_solution_title_desc,
  maybe_vote,
  check_solution,
  maybe_delete,
  new_attempt,
  cancel_new_attempt,
  create_solution,
  submit_solution,
  choose_solution,
  maybe_expand,
};
