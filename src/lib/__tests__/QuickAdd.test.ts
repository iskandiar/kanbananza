import { describe, it, expect } from 'vitest';

/**
 * Extract the URL source detection logic for testing.
 * This mirrors the detectedSource derived value in QuickAdd.svelte
 */
function detectSource(value: string): string | null {
  const LINEAR_URL = /https:\/\/linear\.app\/[^/]+\/issue\/([A-Z]+-\d+)/;
  const NOTION_URL = /https?:\/\/(www\.)?notion\.(so|com)\/.*([a-f0-9]{32})/;
  const SLACK_URL = /https?:\/\/[^.]+\.slack\.com\/archives\/[A-Z0-9]+\/p\d+/;

  return LINEAR_URL.test(value) ? 'Linear issue'
    : NOTION_URL.test(value) ? 'Notion page'
    : SLACK_URL.test(value) ? 'Slack thread'
    : null;
}

describe('QuickAdd URL Source Detection', () => {
  describe('Linear URLs', () => {
    it('detects valid Linear issue URL', () => {
      const url = 'https://linear.app/kanbananza/issue/ENG-123';
      expect(detectSource(url)).toBe('Linear issue');
    });

    it('detects Linear URL with multi-part team slug', () => {
      const url = 'https://linear.app/my-org/issue/PROJ-456';
      expect(detectSource(url)).toBe('Linear issue');
    });

    it('detects Linear URL with complex issue ID', () => {
      const url = 'https://linear.app/workspace/issue/ABC-9999';
      expect(detectSource(url)).toBe('Linear issue');
    });

    it('rejects incomplete Linear URL', () => {
      const url = 'https://linear.app/issue/ENG-123';
      expect(detectSource(url)).toBeNull();
    });

    it('rejects Linear URL without issue ID', () => {
      const url = 'https://linear.app/kanbananza/issue/';
      expect(detectSource(url)).toBeNull();
    });

    it('rejects malformed Linear URL', () => {
      const url = 'https://linear.app/kanbananza/issues/ENG-123';
      expect(detectSource(url)).toBeNull();
    });
  });

  describe('Notion URLs', () => {
    it('detects valid Notion .so URL', () => {
      const url = 'https://www.notion.so/Database-abc123def456789abc123def456789ab';
      expect(detectSource(url)).toBe('Notion page');
    });

    it('detects Notion .so URL without www', () => {
      const url = 'https://notion.so/Page-Title-abc123def456789abc123def456789ab';
      expect(detectSource(url)).toBe('Notion page');
    });

    it('detects Notion .com URL', () => {
      const url = 'https://www.notion.com/My-Page-abc123def456789abc123def456789ab';
      expect(detectSource(url)).toBe('Notion page');
    });

    it('detects Notion .com URL without www', () => {
      const url = 'https://notion.com/abc123def456789abc123def456789ab';
      expect(detectSource(url)).toBe('Notion page');
    });

    it('detects Notion URL with query parameters', () => {
      const url = 'https://notion.so/Database-abc123def456789abc123def456789ab?v=xyz';
      expect(detectSource(url)).toBe('Notion page');
    });

    it('rejects Notion URL with invalid hash (too short)', () => {
      const url = 'https://notion.so/Page-abc123';
      expect(detectSource(url)).toBeNull();
    });

    it('rejects Notion URL with non-hex hash', () => {
      const url = 'https://notion.so/Page-zzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzz';
      expect(detectSource(url)).toBeNull();
    });

    it('rejects non-Notion domain', () => {
      const url = 'https://notary.so/abc123def456789abc123def456789ab';
      expect(detectSource(url)).toBeNull();
    });
  });

  describe('Slack URLs', () => {
    it('detects valid Slack thread URL', () => {
      const url = 'https://my-workspace.slack.com/archives/C1234567890/p1234567890123456';
      expect(detectSource(url)).toBe('Slack thread');
    });

    it('detects Slack URL with different workspace', () => {
      const url = 'https://company-dev.slack.com/archives/C0987654321/p9876543210987654';
      expect(detectSource(url)).toBe('Slack thread');
    });

    it('detects Slack URL with http (not https)', () => {
      const url = 'http://workspace.slack.com/archives/CABCDEF123/p1234567890123456';
      expect(detectSource(url)).toBe('Slack thread');
    });

    it('detects Slack URL with numeric channel ID', () => {
      const url = 'https://test.slack.com/archives/C123456ABC/p1111111111111111';
      expect(detectSource(url)).toBe('Slack thread');
    });

    it('rejects Slack URL with lowercase channel', () => {
      const url = 'https://workspace.slack.com/archives/c1234567890/p1234567890123456';
      expect(detectSource(url)).toBeNull();
    });

    it('rejects Slack URL without timestamp', () => {
      const url = 'https://workspace.slack.com/archives/C1234567890/';
      expect(detectSource(url)).toBeNull();
    });

    it('rejects Slack URL with invalid path', () => {
      const url = 'https://workspace.slack.com/messages/C1234567890';
      expect(detectSource(url)).toBeNull();
    });
  });

  describe('Unknown/Plain text', () => {
    it('returns null for plain text', () => {
      const text = 'Just a regular task description';
      expect(detectSource(text)).toBeNull();
    });

    it('returns null for random URL', () => {
      const url = 'https://example.com/some/path';
      expect(detectSource(url)).toBeNull();
    });

    it('returns null for empty string', () => {
      expect(detectSource('')).toBeNull();
    });

    it('returns null for whitespace', () => {
      expect(detectSource('   ')).toBeNull();
    });

    it('returns null for github URL', () => {
      const url = 'https://github.com/owner/repo/pull/123';
      expect(detectSource(url)).toBeNull();
    });

    it('returns null for jira URL', () => {
      const url = 'https://jira.example.com/browse/PROJ-999';
      expect(detectSource(url)).toBeNull();
    });

    it('returns null for truncated URLs', () => {
      expect(detectSource('https://linear.app')).toBeNull();
      expect(detectSource('https://notion.so')).toBeNull();
      expect(detectSource('https://workspace.slack.com')).toBeNull();
    });
  });

  describe('edge cases and malformed inputs', () => {
    it('handles URL-like text that is not actually a URL', () => {
      const text = 'linear.app/workspace/issue/ENG-123';
      expect(detectSource(text)).toBeNull();
    });

    it('handles URLs with special characters in title', () => {
      const url = 'https://www.notion.so/My-Page-with-Special-Chars-!@#-abc123def456789abc123def456789ab';
      expect(detectSource(url)).toBe('Notion page');
    });

    it('handles mixed case in domain', () => {
      const url = 'HTTPS://LINEAR.APP/workspace/issue/ENG-123';
      expect(detectSource(url)).toBeNull(); // regex is case-sensitive
    });

    it('handles URLs with fragments', () => {
      const url = 'https://linear.app/workspace/issue/ENG-123#comment-xyz';
      expect(detectSource(url)).toBe('Linear issue');
    });

    it('handles Slack URL with extra path segments', () => {
      const url = 'https://workspace.slack.com/archives/C1234567890/p1234567890123456/more/path';
      expect(detectSource(url)).toBe('Slack thread');
    });
  });
});
