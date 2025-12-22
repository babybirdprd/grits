import { render, screen } from '@testing-library/react';
import { ListView } from '../src/components/ListView';
import { describe, it, expect, vi } from 'vitest';

// Mock AutoSizer to render children with fixed dimensions
vi.mock('react-virtualized-auto-sizer', () => ({
  default: ({ children }: any) => children({ height: 500, width: 500 })
}));

const mockIssue = {
    id: "123",
    content_hash: "abc",
    title: "Test Issue",
    description: "Desc",
    design: "",
    acceptance_criteria: "",
    notes: "",
    status: "open",
    priority: 1,
    issue_type: "bug",
    assignee: "me",
    estimated_minutes: null,
    created_at: new Date().toISOString(),
    updated_at: new Date().toISOString(),
    closed_at: null,
    external_ref: null,
    sender: "me",
    ephemeral: false,
    replies_to: "",
    relates_to: [],
    duplicate_of: "",
    superseded_by: "",
    deleted_at: null,
    deleted_by: "",
    delete_reason: "",
    original_type: "",
    labels: ["urgent"],
    dependencies: [],
    comments: []
};

describe('ListView', () => {
  it('renders issue list', () => {
    render(
      <ListView
        issues={[mockIssue]}
        onUpdateField={() => {}}
        onSelectIssue={() => {}}
      />
    );
    expect(screen.getByText('Test Issue')).toBeDefined();
    expect(screen.getByText('urgent')).toBeDefined();
  });
});
