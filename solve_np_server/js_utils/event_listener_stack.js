"use strict";
class EventListenerStack {
    constructor(target, type) {
        this.event_listener_stack = [];
        this.target = target;
        this.type = type;
    }
    push(event_listener) {
        const current_listener = this.event_listener_stack.at(-1);
        if (current_listener) {
            this.target.removeEventListener(this.type, current_listener);
        }
        this.event_listener_stack.push(event_listener);
        this.target.addEventListener(this.type, event_listener);
    }
    pop() {
        const current_listener = this.event_listener_stack.pop();
        if (current_listener) {
            this.target.removeEventListener(this.type, current_listener);
            const previous_listener = this.event_listener_stack.at(-1);
            if (previous_listener) {
                this.target.addEventListener(this.type, previous_listener);
            }
        }
    }
}
const global_event_listener_stack = new EventListenerStack(window, 'keydown');
