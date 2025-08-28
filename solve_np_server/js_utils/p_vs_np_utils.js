var PvsNp;
(function (PvsNp) {
    PvsNp[PvsNp["Eq"] = 0] = "Eq";
    PvsNp[PvsNp["Neq"] = 1] = "Neq";
})(PvsNp || (PvsNp = {}));
function vote(choice) {
    fetch('/api/vote/' + choice, { method: 'POST' })
        .then(function () { return window.location.reload(); })
        .catch(console.log);
}
