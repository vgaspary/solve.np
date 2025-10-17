"use strict";
function parse_solution(problem_id, input) {
    if (problem_id === "travelling_salesman") {
        return { TspSolution: input.trim().split(/\s+/).map(n => Number(n)) };
    }
    else if (problem_id === "graph_coloring") {
        return { GraphColoringSolution: input.trim().split(/\s+/).map(n => Number(n)) };
    }
    return null;
}
function check_solution(problem_id) {
    const solution_textarea = document.getElementById("sample_solution");
    if (!solution_textarea) {
        throw new Error("'sample_solution' textarea not found.");
    }
    const solution = solution_textarea.value;
    const sample_solution_result = document.getElementById('sample_solution_result');
    if (!sample_solution_result) {
        throw new Error("'sample_solution' textarea not found.");
    }
    if (solution === "") {
        solution_textarea.classList = "";
        sample_solution_result.innerHTML = "";
        return;
    }
    fetch('/api/validate_sample', {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json'
        },
        body: JSON.stringify(parse_solution(problem_id, solution))
    })
        .then(response => response.json())
        .then(validation_result => {
        if (validation_result.Valid) {
            solution_textarea.classList = "success";
            sample_solution_result.innerHTML = validation_result.Valid;
        }
        else if (validation_result.Invalid) {
            solution_textarea.classList = "error";
            sample_solution_result.innerHTML = validation_result.Invalid;
        }
    })
        .catch(err => {
        solution_textarea.classList = "";
        sample_solution_result.innerHTML = "";
        console.log(err);
    });
}
function submit_solution(problem_id, solution_id, input_id) {
    const solution_textarea = document.getElementById(input_id + '_solution');
    if (!solution_textarea) {
        throw new Error(input_id + '_solution textarea not found.');
    }
    solution_textarea.classList = "";
    const solution = solution_textarea.value;
    if (solution.trim() === '') {
        return;
    }
    const status_paragraph = document.getElementById(input_id + '_status');
    if (!status_paragraph) {
        throw new Error(input_id + '_status paragraph not found.');
    }
    const status_label = document.getElementById(input_id + '_status_label');
    if (!status_label) {
        throw new Error(input_id + '_status_label span not found.');
    }
    const cancellation_token = { stop: false };
    show_in_progress(status_paragraph, 0, cancellation_token);
    fetch('/api/submit_solution/' + problem_id + '/' + solution_id + '/' + input_id, {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json'
        },
        body: JSON.stringify(parse_solution(problem_id, solution))
    })
        .then(response => {
        cancellation_token.stop = true;
        if (response.status === 204) {
            status_paragraph.innerHTML = "This solution has already been submitted.";
            solution_textarea.classList = "";
            return;
        }
        return response.json();
    })
        .then(validation_result => {
        if (validation_result.Valid) {
            status_paragraph.innerHTML = 'Score: ' + validation_result.Valid;
            solution_textarea.classList = "success";
            status_label.innerHTML = 'Solved';
            status_label.style = 'color: green; width: 130px; display: inline-block;';
        }
        else if (validation_result.Invalid) {
            status_paragraph.innerHTML = validation_result.Invalid;
            solution_textarea.classList = "error";
            status_label.innerHTML = 'Attempted';
            status_label.style = 'color: orange; width: 130px; display: inline-block;';
        }
    })
        .catch(err => {
        cancellation_token.stop = true;
        console.log(err);
    });
}
function show_in_progress(paragraph, i, cancellation_token) {
    if (cancellation_token.stop) {
        return;
    }
    const progress = '⠋⠙⠸⢰⣠⣄⡆⠇';
    paragraph.innerHTML = progress[i % progress.length] + ' Validation in progress...';
    setTimeout(() => show_in_progress(paragraph, i + 1, cancellation_token), 100);
}
