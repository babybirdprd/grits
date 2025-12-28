// Example: TypeScript module with circular dependency
// This file imports helper, and helper imports this file back

import { formatOutput, ValidationResult } from './helper';

interface DataRecord {
    id: string;
    name: string;
    timestamp: Date;
    metadata: Record<string, unknown>;
}

class DataService {
    private records: DataRecord[] = [];
    private cache: Map<string, DataRecord> = new Map();

    constructor(private readonly maxCacheSize: number = 100) {}

    addRecord(record: DataRecord): void {
        const formatted = formatOutput(record.name);
        this.records.push({ ...record, name: formatted });
        this.updateCache(record);
    }

    private updateCache(record: DataRecord): void {
        if (this.cache.size >= this.maxCacheSize) {
            const firstKey = this.cache.keys().next().value;
            this.cache.delete(firstKey);
        }
        this.cache.set(record.id, record);
    }

    getRecord(id: string): DataRecord | undefined {
        return this.cache.get(id) || this.records.find(r => r.id === id);
    }

    getStats(): ServiceStats {
        return {
            totalRecords: this.records.length,
            cacheHitRate: this.cache.size / Math.max(this.records.length, 1),
        };
    }
}

interface ServiceStats {
    totalRecords: number;
    cacheHitRate: number;
}

// Function that helper.ts will call back to, creating a cycle
export function validateService(service: DataService): ValidationResult {
    const stats = service.getStats();
    return {
        isValid: stats.totalRecords > 0,
        message: `Service has ${stats.totalRecords} records`,
    };
}

export { DataService, DataRecord, ServiceStats };
