// Example: JavaScript util that creates a circular dependency
// This file imports main (validateEventBus), and main imports this file (processPayload)

import { validateEventBus, EventBus } from './main';

export function processPayload(payload) {
    if (typeof payload === 'string') {
        return { data: payload, type: 'string', processed: true };
    }
    if (typeof payload === 'object' && payload !== null) {
        return { ...payload, processed: true, timestamp: Date.now() };
    }
    return { data: payload, type: typeof payload, processed: true };
}

export function createLogger(namespace) {
    const prefix = `[${namespace}]`;
    return {
        info: (msg) => console.log(`${prefix} INFO: ${msg}`),
        warn: (msg) => console.warn(`${prefix} WARN: ${msg}`),
        error: (msg) => console.error(`${prefix} ERROR: ${msg}`),
        debug: (msg) => console.debug(`${prefix} DEBUG: ${msg}`),
    };
}

export function batchValidate(buses) {
    // This creates a cycle: util -> main (validateEventBus) -> util (processPayload)
    return buses.map(bus => {
        const validation = validateEventBus(bus);
        return {
            ...validation,
            payload: processPayload(validation),
        };
    });
}

export class PayloadTransformer {
    constructor(transforms = []) {
        this.transforms = transforms;
    }

    addTransform(fn) {
        this.transforms.push(fn);
    }

    apply(payload) {
        let result = processPayload(payload);
        for (const transform of this.transforms) {
            result = transform(result);
        }
        return result;
    }
}
