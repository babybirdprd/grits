// Example: JavaScript module with circular dependency
// This file imports util, and util imports this file back

import { processPayload, createLogger } from './util';

class EventBus {
    constructor() {
        this.listeners = new Map();
        this.eventHistory = [];
        this.logger = createLogger('EventBus');
    }

    subscribe(eventType, callback) {
        if (!this.listeners.has(eventType)) {
            this.listeners.set(eventType, []);
        }
        this.listeners.get(eventType).push(callback);
        this.logger.info(`Subscribed to ${eventType}`);
    }

    publish(eventType, payload) {
        const processed = processPayload(payload);
        this.eventHistory.push({
            type: eventType,
            payload: processed,
            timestamp: Date.now(),
        });

        const callbacks = this.listeners.get(eventType) || [];
        callbacks.forEach(cb => cb(processed));
    }

    getHistory() {
        return [...this.eventHistory];
    }
}

class StateManager {
    constructor(eventBus) {
        this.state = {};
        this.eventBus = eventBus;
        this.subscribers = [];
    }

    setState(key, value) {
        const oldValue = this.state[key];
        this.state[key] = value;

        this.eventBus.publish('state:change', {
            key,
            oldValue,
            newValue: value,
        });

        this.notifySubscribers(key, value);
    }

    getState(key) {
        return this.state[key];
    }

    notifySubscribers(key, value) {
        this.subscribers.forEach(sub => sub(key, value));
    }
}

// Function that util.js will call back to, creating a cycle
export function validateEventBus(bus) {
    const history = bus.getHistory();
    return {
        isValid: history.length >= 0,
        eventCount: history.length,
        listenerCount: bus.listeners.size,
    };
}

export { EventBus, StateManager };
