import { describe, it, expect } from 'vitest';
import { extractFromTitle } from '../components/QuickAdd.svelte';

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

describe('extractFromTitle', () => {
  describe('H:MM time format', () => {
    it('parses 0:30 as 0.5h', () => {
      const result = extractFromTitle('Fix login bug 0:30');
      expect(result.timeEstimate).toBeCloseTo(0.5);
      expect(result.cleanedTitle).toBe('Fix login bug');
    });
    it('parses 1:30 as 1.5h', () => {
      const result = extractFromTitle('Deploy service 1:30');
      expect(result.timeEstimate).toBeCloseTo(1.5);
      expect(result.cleanedTitle).toBe('Deploy service');
    });
    it('parses 2:00 as 2h', () => {
      const result = extractFromTitle('Write tests 2:00');
      expect(result.timeEstimate).toBeCloseTo(2.0);
      expect(result.cleanedTitle).toBe('Write tests');
    });
    it('parses 0:20h as 20 minutes (trailing h ignored)', () => {
      const result = extractFromTitle('Task 0:20h');
      expect(result.timeEstimate).toBeCloseTo(1 / 3);
      expect(result.cleanedTitle).toBe('Task');
    });
    it('parses 1:30h as 1.5h (trailing h ignored)', () => {
      const result = extractFromTitle('Task 1:30h');
      expect(result.timeEstimate).toBeCloseTo(1.5);
      expect(result.cleanedTitle).toBe('Task');
    });
  });

  describe('existing time formats still work', () => {
    it('parses 1h', () => {
      const result = extractFromTitle('Fix bug 1h');
      expect(result.timeEstimate).toBe(1);
      expect(result.cleanedTitle).toBe('Fix bug');
    });
    it('parses 30m', () => {
      const result = extractFromTitle('Code review 30m');
      expect(result.timeEstimate).toBeCloseTo(0.5);
      expect(result.cleanedTitle).toBe('Code review');
    });
    it('parses 1.5h', () => {
      const result = extractFromTitle('Planning 1.5h');
      expect(result.timeEstimate).toBe(1.5);
      expect(result.cleanedTitle).toBe('Planning');
    });
  });

  describe('URL extraction', () => {
    it('extracts URL from mixed input', () => {
      const result = extractFromTitle('Fix login https://example.com 1h');
      expect(result.url).toBe('https://example.com');
      expect(result.timeEstimate).toBe(1);
      expect(result.cleanedTitle).toBe('Fix login');
    });
  });

  describe('priority extraction', () => {
    it('extracts #high', () => {
      const result = extractFromTitle('Important task #high');
      expect(result.impact).toBe('high');
      expect(result.cleanedTitle).toBe('Important task');
      expect(result.cardType).toBeNull();
    });
    it('extracts #mid', () => {
      const result = extractFromTitle('Task #mid');
      expect(result.impact).toBe('mid');
      expect(result.cleanedTitle).toBe('Task');
    });
    it('extracts #low', () => {
      const result = extractFromTitle('Task #low');
      expect(result.impact).toBe('low');
      expect(result.cleanedTitle).toBe('Task');
    });
    it('extracts #h shorthand', () => {
      const result = extractFromTitle('Critical fix #h');
      expect(result.impact).toBe('high');
      expect(result.cleanedTitle).toBe('Critical fix');
    });
    it('extracts #m shorthand', () => {
      const result = extractFromTitle('Task #m');
      expect(result.impact).toBe('mid');
    });
    it('extracts #l shorthand', () => {
      const result = extractFromTitle('Task #l');
      expect(result.impact).toBe('low');
    });
    it('last #priority wins', () => {
      const result = extractFromTitle('Deploy fix #high #low');
      expect(result.impact).toBe('low');
      expect(result.cleanedTitle).toBe('Deploy fix');
    });
    it('does not extract !priority (old syntax removed)', () => {
      const result = extractFromTitle('Important task !high');
      expect(result.impact).toBeNull();
      expect(result.cleanedTitle).toBe('Important task !high');
    });
  });

  describe('card type extraction', () => {
    it('extracts #task', () => {
      const result = extractFromTitle('Fix bug #task');
      expect(result.cardType).toBe('task');
      expect(result.cleanedTitle).toBe('Fix bug');
    });
    it('extracts #todo as task', () => {
      const result = extractFromTitle('Fix bug #todo');
      expect(result.cardType).toBe('task');
      expect(result.cleanedTitle).toBe('Fix bug');
    });
    it('extracts #meeting', () => {
      const result = extractFromTitle('Standup #meeting');
      expect(result.cardType).toBe('meeting');
      expect(result.cleanedTitle).toBe('Standup');
    });
    it('extracts #meet as meeting', () => {
      const result = extractFromTitle('Standup #meet');
      expect(result.cardType).toBe('meeting');
    });
    it('extracts #mr', () => {
      const result = extractFromTitle('Review auth PR #mr');
      expect(result.cardType).toBe('mr');
      expect(result.cleanedTitle).toBe('Review auth PR');
    });
    it('does not confuse #mr with #m (priority mid)', () => {
      const result = extractFromTitle('Task #mr #m');
      expect(result.cardType).toBe('mr');
      expect(result.impact).toBe('mid');
    });
    it('extracts #thread', () => {
      const result = extractFromTitle('Slack convo #thread');
      expect(result.cardType).toBe('thread');
    });
    it('extracts #review', () => {
      const result = extractFromTitle('Code review #review');
      expect(result.cardType).toBe('review');
    });
    it('extracts #doc', () => {
      const result = extractFromTitle('Write ADR #doc');
      expect(result.cardType).toBe('documentation');
    });
    it('extracts #documentation', () => {
      const result = extractFromTitle('Write ADR #documentation');
      expect(result.cardType).toBe('documentation');
    });
    it('last #type wins', () => {
      const result = extractFromTitle('Task #mr #task');
      expect(result.cardType).toBe('task');
    });
    it('returns null for no type token', () => {
      const result = extractFromTitle('Just a task');
      expect(result.cardType).toBeNull();
    });
    it('leaves unrecognised #words in title', () => {
      const result = extractFromTitle('Notes about #project planning');
      expect(result.cardType).toBeNull();
      expect(result.cleanedTitle).toBe('Notes about #project planning');
    });
  });

  describe('combined input', () => {
    it('handles URL + time + type + priority together', () => {
      const result = extractFromTitle('Fix auth timeout https://gitlab.local/mr/42 0:30 #mr #high');
      expect(result.url).toBe('https://gitlab.local/mr/42');
      expect(result.timeEstimate).toBeCloseTo(0.5);
      expect(result.impact).toBe('high');
      expect(result.cardType).toBe('mr');
      expect(result.cleanedTitle).toBe('Fix auth timeout');
    });
    it('#high inside URL fragment is not parsed as priority token', () => {
      const result = extractFromTitle('Task https://example.com/#high');
      expect(result.url).toBe('https://example.com/#high');
      expect(result.impact).toBeNull();
      expect(result.cleanedTitle).toBe('Task');
    });
    it('type and priority tokens are case-insensitive', () => {
      const result = extractFromTitle('Fix bug #MR #HIGH');
      expect(result.cardType).toBe('mr');
      expect(result.impact).toBe('high');
      expect(result.cleanedTitle).toBe('Fix bug');
    });
  });
});
