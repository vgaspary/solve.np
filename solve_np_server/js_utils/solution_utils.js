var __awaiter = (this && this.__awaiter) || function (thisArg, _arguments, P, generator) {
    function adopt(value) { return value instanceof P ? value : new P(function (resolve) { resolve(value); }); }
    return new (P || (P = Promise))(function (resolve, reject) {
        function fulfilled(value) { try { step(generator.next(value)); } catch (e) { reject(e); } }
        function rejected(value) { try { step(generator["throw"](value)); } catch (e) { reject(e); } }
        function step(result) { result.done ? resolve(result.value) : adopt(result.value).then(fulfilled, rejected); }
        step((generator = generator.apply(thisArg, _arguments || [])).next());
    });
};
var __generator = (this && this.__generator) || function (thisArg, body) {
    var _ = { label: 0, sent: function() { if (t[0] & 1) throw t[1]; return t[1]; }, trys: [], ops: [] }, f, y, t, g = Object.create((typeof Iterator === "function" ? Iterator : Object).prototype);
    return g.next = verb(0), g["throw"] = verb(1), g["return"] = verb(2), typeof Symbol === "function" && (g[Symbol.iterator] = function() { return this; }), g;
    function verb(n) { return function (v) { return step([n, v]); }; }
    function step(op) {
        if (f) throw new TypeError("Generator is already executing.");
        while (g && (g = 0, op[0] && (_ = 0)), _) try {
            if (f = 1, y && (t = op[0] & 2 ? y["return"] : op[0] ? y["throw"] || ((t = y["return"]) && t.call(y), 0) : y.next) && !(t = t.call(y, op[1])).done) return t;
            if (y = 0, t) op = [op[0] & 2, t.value];
            switch (op[0]) {
                case 0: case 1: t = op; break;
                case 4: _.label++; return { value: op[1], done: false };
                case 5: _.label++; y = op[1]; op = [0]; continue;
                case 7: op = _.ops.pop(); _.trys.pop(); continue;
                default:
                    if (!(t = _.trys, t = t.length > 0 && t[t.length - 1]) && (op[0] === 6 || op[0] === 2)) { _ = 0; continue; }
                    if (op[0] === 3 && (!t || (op[1] > t[0] && op[1] < t[3]))) { _.label = op[1]; break; }
                    if (op[0] === 6 && _.label < t[1]) { _.label = t[1]; t = op; break; }
                    if (t && _.label < t[2]) { _.label = t[2]; _.ops.push(op); break; }
                    if (t[2]) _.ops.pop();
                    _.trys.pop(); continue;
            }
            op = body.call(thisArg, _);
        } catch (e) { op = [6, e]; y = 0; } finally { f = t = 0; }
        if (op[0] & 5) throw op[1]; return { value: op[0] ? op[1] : void 0, done: true };
    }
};
function maybe_delete(problem_id, solution_id, solution_title) {
    var dialog = document.getElementById('confirm-dialog');
    var message = document.getElementById('confirm-dialog-message');
    var confirmButton = document.getElementById('confirm-dialog-button');
    var cancelButton = document.getElementById('cancel-dialog-button');
    if (dialog && message && confirmButton && cancelButton) {
        message.textContent = "Are you sure you want to delete the ".concat(solution_title, " solution?");
        dialog.style.display = 'flex';
        confirmButton.onclick = function () {
            delete_solution(problem_id, solution_id);
            dialog.style.display = 'none';
        };
        cancelButton.onclick = function () {
            dialog.style.display = 'none';
        };
    }
}
function delete_solution(problem_id, solution_id) {
    fetch("/api/solutions/".concat(problem_id, "/").concat(solution_id), { method: 'DELETE' })
        .then(function () {
        location.reload();
    })
        .catch(console.log);
}
function new_attempt(button) {
    var table = button.closest('table');
    var new_attempt_row = table.querySelector('#new_attempt_row');
    var create_cancel_row = table.querySelector('#create_cancel_row');
    var new_row = table.insertRow(table.rows.length - 2);
    new_row.innerHTML = "\n    <td></td>\n    <td></td>\n    <td><input type=\"text\" autocomplete=\"off\" placeholder=\"Title\" id=\"new_title\"/></td>\n    <td><input type=\"text\" autocomplete=\"off\" placeholder=\"Description (optional)\" id=\"new_description\"/></td>\n  ";
    new_attempt_row.style.display = 'none';
    create_cancel_row.style.display = 'contents';
}
function cancel_new_attempt(button) {
    var table = button.closest('table');
    var new_attempt_row = table.querySelector('#new_attempt_row');
    var create_cancel_row = table.querySelector('#create_cancel_row');
    var error_msg = table.querySelector('#create_attempt_error_message');
    table.deleteRow(table.rows.length - 3);
    new_attempt_row.style.display = 'contents';
    create_cancel_row.style.display = 'none';
    error_msg.innerHTML = '';
}
function create_solution(button, problemId) {
    var _this = this;
    var table = button.closest('table');
    var title = table.querySelector('#new_title');
    var description = table.querySelector('#new_description');
    var error_msg = table.querySelector('#create_attempt_error_message');
    if (title.value.trim() === '') {
        error_msg.innerHTML = '\'Title\' field is mandatory.';
        return;
    }
    fetch('/api/solutions', {
        method: 'POST',
        body: JSON.stringify({ problem_id: problemId, title: title.value, description: description.value }),
        headers: {
            'Content-Type': 'application/json',
        },
    })
        .then(function (response) { return __awaiter(_this, void 0, void 0, function () {
        return __generator(this, function (_a) {
            switch (_a.label) {
                case 0:
                    if (!response.ok) return [3 /*break*/, 1];
                    location.reload();
                    return [3 /*break*/, 3];
                case 1: return [4 /*yield*/, response.text()];
                case 2: throw _a.sent();
                case 3: return [2 /*return*/];
            }
        });
    }); })
        .catch(function (error) {
        error_msg.innerHTML = "".concat(error);
    });
}
