// Example: TypeScript helper that creates a circular dependency
// This file imports index (validateService), and index imports this file (formatOutput)

import { validateService, DataService } from './index';

export interface ValidationResult {
    isValid: boolean;
    message: string;
}

export function formatOutput(input: string): string {
    // Simulate formatting logic
    return input
        .trim()
        .toLowerCase()
        .replace(/[^a-z0-9\s]/g, '')
        .split(/\s+/)
        .map(word => word.charAt(0).toUpperCase() + word.slice(1))
        .join(' ');
}

export function batchValidate(services: DataService[]): ValidationResult[] {
    // This creates a cycle: helper -> index (validateService) -> helper (formatOutput)
    return services.map(service => {
        const result = validateService(service);
        return {
            ...result,
            message: formatOutput(result.message),
        };
    });
}

export class OutputFormatter {
    private prefix: string;
    private suffix: string;

    constructor(prefix: string = '[', suffix: string = ']') {
        this.prefix = prefix;
        this.suffix = suffix;
    }

    format(value: string): string {
        return `${this.prefix}${formatOutput(value)}${this.suffix}`;
    }
}
