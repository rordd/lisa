import { useMemo, useRef, useEffect, useCallback, useState } from 'react';

interface Props {
  rawToml: string;
  onChange: (raw: string) => void;
  disabled?: boolean;
}

// Token types for TOML syntax highlighting
type TokenType = 'comment' | 'section' | 'key' | 'string' | 'number' | 'boolean' | 'datetime' | 'punctuation';

interface Token {
  type: TokenType;
  value: string;
}

// Tokenizer for TOML
function tokenizeToml(code: string): Token[][] {
  const lines = code.split('\n');
  return lines.map(line => {
    const tokens: Token[] = [];
    let remaining = line;

    while (remaining.length > 0) {
      // Try to match patterns at the start of remaining string

      // Comment (# to end of line)
      if (remaining.startsWith('#')) {
        tokens.push({ type: 'comment', value: remaining });
        remaining = '';
        continue;
      }

      // Section header [section] or [section.subsection]
      const sectionMatch = remaining.match(/^\s*(\[+[^\]]+\]+)(\s*)$/);
      if (sectionMatch && tokens.length === 0) {
        const sectionValue = sectionMatch[1];
        if (sectionValue) {
          const leadingSpaces = remaining.match(/^(\s*)/)?.[1] ?? '';
          if (leadingSpaces) {
            tokens.push({ type: 'punctuation', value: leadingSpaces });
          }
          tokens.push({ type: 'section', value: sectionValue });
        }
        const trailingSpaces = sectionMatch[2];
        if (trailingSpaces) {
          tokens.push({ type: 'comment', value: trailingSpaces });
        }
        remaining = '';
        continue;
      }

      // Key = value pattern
      const keyValueMatch = remaining.match(/^(\s*)([a-zA-Z_][a-zA-Z0-9_\-]*)(\s*=\s*)/);
      if (keyValueMatch) {
        const leadingWs = keyValueMatch[1];
        const keyName = keyValueMatch[2];
        const equalsWs = keyValueMatch[3];
        const fullMatch = keyValueMatch[0];

        if (leadingWs) {
          tokens.push({ type: 'punctuation', value: leadingWs });
        }
        if (keyName) {
          tokens.push({ type: 'key', value: keyName });
        }
        if (equalsWs) {
          tokens.push({ type: 'punctuation', value: equalsWs });
        }
        remaining = remaining.slice(fullMatch.length);

        // Now parse the value
        const valuePart = remaining;

        // String (double-quoted)
        const stringDoubleMatch = valuePart.match(/^"(?:[^"\\]|\\.)*"/);
        if (stringDoubleMatch?.[0]) {
          tokens.push({ type: 'string', value: stringDoubleMatch[0] });
          remaining = valuePart.slice(stringDoubleMatch[0].length);
          continue;
        }

        // String (single-quoted/literal)
        const stringSingleMatch = valuePart.match(/^'(?:[^'\\]|\\.)*'/);
        if (stringSingleMatch?.[0]) {
          tokens.push({ type: 'string', value: stringSingleMatch[0] });
          remaining = valuePart.slice(stringSingleMatch[0].length);
          continue;
        }

        // Multi-line strings """ ... """ and ''' ... '''
        const multiStringMatch = valuePart.match(/^"{3}(?:[^"\\]|\\.)*"{3}|^'{3}(?:[^'\\]|\\.)*'{3}/);
        if (multiStringMatch?.[0]) {
          tokens.push({ type: 'string', value: multiStringMatch[0] });
          remaining = valuePart.slice(multiStringMatch[0].length);
          continue;
        }

        // Boolean
        const boolMatch = valuePart.match(/^(true|false)(?=[\s,\]\}#]|$)/);
        if (boolMatch?.[1]) {
          tokens.push({ type: 'boolean', value: boolMatch[1] });
          remaining = valuePart.slice(boolMatch[0].length);
          continue;
        }

        // Datetime
        const datetimeMatch = valuePart.match(/^\d{4}-\d{2}-\d{2}(?:[T\s]\d{2}:\d{2}:\d{2}(?:\.\d+)?(?:Z|[+-]\d{2}:\d{2})?)?(?=[\s,\]\}#]|$)/);
        if (datetimeMatch?.[0]) {
          tokens.push({ type: 'datetime', value: datetimeMatch[0] });
          remaining = valuePart.slice(datetimeMatch[0].length);
          continue;
        }

        // Hex number
        const hexMatch = valuePart.match(/^0x[0-9a-fA-F]+(?=[\s,\]\}#]|$)/);
        if (hexMatch?.[0]) {
          tokens.push({ type: 'number', value: hexMatch[0] });
          remaining = valuePart.slice(hexMatch[0].length);
          continue;
        }

        // Octal number
        const octalMatch = valuePart.match(/^0o[0-7]+(?=[\s,\]\}#]|$)/);
        if (octalMatch?.[0]) {
          tokens.push({ type: 'number', value: octalMatch[0] });
          remaining = valuePart.slice(octalMatch[0].length);
          continue;
        }

        // Binary number
        const binaryMatch = valuePart.match(/^0b[01]+(?=[\s,\]\}#]|$)/);
        if (binaryMatch?.[0]) {
          tokens.push({ type: 'number', value: binaryMatch[0] });
          remaining = valuePart.slice(binaryMatch[0].length);
          continue;
        }

        // Float/Integer
        const numMatch = valuePart.match(/^-?(?:0|[1-9]\d*)(?:\.\d+)?(?:[eE][+-]?\d+)?(?=[\s,\]\}#]|$)/);
        if (numMatch?.[0]) {
          tokens.push({ type: 'number', value: numMatch[0] });
          remaining = valuePart.slice(numMatch[0].length);
          continue;
        }

        // Array or inline table brackets
        const firstChar = valuePart[0];
        if (firstChar === '[' || firstChar === ']' || firstChar === '{' || firstChar === '}') {
          tokens.push({ type: 'punctuation', value: firstChar });
          remaining = valuePart.slice(1);
          continue;
        }

        // Comma
        if (valuePart.startsWith(',')) {
          tokens.push({ type: 'punctuation', value: ',' });
          remaining = valuePart.slice(1);
          continue;
        }

        // Whitespace in value
        const wsMatch = valuePart.match(/^(\s+)/);
        if (wsMatch?.[1]) {
          tokens.push({ type: 'punctuation', value: wsMatch[1] });
          remaining = valuePart.slice(wsMatch[1].length);
          continue;
        }

        // Any other character
        const nextChar = valuePart[0];
        if (nextChar) {
          tokens.push({ type: 'punctuation', value: nextChar });
          remaining = valuePart.slice(1);
        } else {
          break;
        }
        continue;
      }

      // Whitespace at line start (no key-value)
      const wsMatch = remaining.match(/^(\s+)/);
      if (wsMatch?.[1]) {
        tokens.push({ type: 'punctuation', value: wsMatch[1] });
        remaining = remaining.slice(wsMatch[1].length);
        continue;
      }

      // Any other character
      const nextChar = remaining[0];
      if (nextChar) {
        tokens.push({ type: 'punctuation', value: nextChar });
        remaining = remaining.slice(1);
      } else {
        break;
      }
    }

    return tokens;
  });
}

// Component to render highlighted line
function HighlightedLine({ tokens }: { tokens: Token[] }) {
  return (
    <span>
      {tokens.map((token, i) => (
        <span key={i} className={`toml-${token.type}`}>
          {token.value}
        </span>
      ))}
    </span>
  );
}

// Line numbers component
function LineNumbers({ count, scrollTop }: { count: number; scrollTop: number }) {
  return (
    <div
      className="flex flex-col text-right select-none text-gray-500 font-mono text-sm leading-[1.5] py-4 pr-3"
      style={{ transform: `translateY(-${scrollTop}px)` }}
      aria-hidden="true"
    >
      {Array.from({ length: count }, (_, i) => (
        <div key={i + 1} className="min-h-[1.5em]">
          {i + 1}
        </div>
      ))}
    </div>
  );
}

export default function ConfigRawEditor({ rawToml, onChange, disabled }: Props) {
  const textareaRef = useRef<HTMLTextAreaElement>(null);
  const highlightRef = useRef<HTMLDivElement>(null);
  const [scrollTop, setScrollTop] = useState(0);
  const [isFocused, setIsFocused] = useState(false);

  const lineCount = useMemo(() => rawToml.split('\n').length, [rawToml]);
  const tokenizedLines = useMemo(() => tokenizeToml(rawToml), [rawToml]);

  const handleScroll = useCallback((e: React.UIEvent<HTMLTextAreaElement>) => {
    const newScrollTop = e.currentTarget.scrollTop;
    setScrollTop(newScrollTop);
    if (highlightRef.current) {
      highlightRef.current.scrollTop = newScrollTop;
      highlightRef.current.scrollLeft = e.currentTarget.scrollLeft;
    }
  }, []);

  const handleKeyDown = useCallback((e: React.KeyboardEvent<HTMLTextAreaElement>) => {
    // Handle Tab key for proper indentation
    if (e.key === 'Tab') {
      e.preventDefault();
      const textarea = textareaRef.current;
      if (!textarea) return;

      const start = textarea.selectionStart;
      const end = textarea.selectionEnd;
      const value = textarea.value;

      if (e.shiftKey) {
        // Shift+Tab: remove indentation
        const lineStart = value.lastIndexOf('\n', start - 1) + 1;
        const lineContent = value.slice(lineStart, start);
        const indentMatch = lineContent.match(/^(\t| {1,4})/);
        if (indentMatch?.[1]) {
          const indentLen = indentMatch[1].length;
          const newValue = value.slice(0, lineStart) + value.slice(lineStart + indentLen);
          onChange(newValue);
          requestAnimationFrame(() => {
            textarea.selectionStart = textarea.selectionEnd = start - indentLen;
          });
        }
      } else {
        // Tab: add indentation
        const newValue = value.slice(0, start) + '    ' + value.slice(end);
        onChange(newValue);
        requestAnimationFrame(() => {
          textarea.selectionStart = textarea.selectionEnd = start + 4;
        });
      }
    }
  }, [onChange]);

  // Sync scroll between textarea and highlight layer
  useEffect(() => {
    const textarea = textareaRef.current;
    const highlight = highlightRef.current;
    if (!textarea || !highlight) return;

    const syncScroll = () => {
      highlight.scrollTop = textarea.scrollTop;
      highlight.scrollLeft = textarea.scrollLeft;
    };

    textarea.addEventListener('scroll', syncScroll);
    return () => textarea.removeEventListener('scroll', syncScroll);
  }, []);

  return (
    <div className="bg-gray-900 rounded-xl border border-gray-800 overflow-hidden">
      {/* Header */}
      <div className="flex items-center justify-between px-4 py-2 border-b border-gray-800 bg-gray-800/50">
        <span className="text-xs text-gray-400 font-medium uppercase tracking-wider">
          TOML Configuration
        </span>
        <div className="flex items-center gap-4">
          <span className="text-xs text-gray-500">
            {lineCount} {lineCount === 1 ? 'line' : 'lines'}
          </span>
          {isFocused && (
            <span className="text-xs text-blue-400">
              Editing
            </span>
          )}
        </div>
      </div>

      {/* Editor container with line numbers */}
      <div className="relative flex bg-gray-950">
        {/* Line numbers column */}
        <div
          className={`sticky left-0 z-10 bg-gray-950 border-r border-gray-800 overflow-hidden ${
            isFocused ? 'bg-gray-900/50' : ''
          }`}
          style={{ width: '3.5rem', minWidth: '3.5rem' }}
        >
          <LineNumbers count={lineCount} scrollTop={scrollTop} />
        </div>

        {/* Code area with syntax highlighting overlay */}
        <div className="relative flex-1 overflow-hidden">
          {/* Highlighted code (background layer) */}
          <div
            ref={highlightRef}
            className="absolute inset-0 pointer-events-none overflow-auto font-mono text-sm leading-[1.5] py-4 pl-4 pr-4 whitespace-pre"
            style={{
              tabSize: 4,
              MozTabSize: 4,
            }}
            aria-hidden="true"
          >
            {tokenizedLines.map((tokens, i) => (
              <div key={i} className="min-h-[1.5em]">
                <HighlightedLine tokens={tokens} />
              </div>
            ))}
          </div>

          {/* Textarea (foreground layer, transparent text) */}
          <textarea
            ref={textareaRef}
            value={rawToml}
            onChange={(e) => onChange(e.target.value)}
            onScroll={handleScroll}
            onKeyDown={handleKeyDown}
            onFocus={() => setIsFocused(true)}
            onBlur={() => setIsFocused(false)}
            disabled={disabled}
            spellCheck={false}
            aria-label="Raw TOML configuration editor"
            className="relative z-10 w-full min-h-[500px] bg-transparent text-transparent caret-white font-mono text-sm leading-[1.5] py-4 pl-4 pr-4 resize-y focus:outline-none disabled:opacity-50"
            style={{
              tabSize: 4,
              MozTabSize: 4,
            }}
          />
        </div>
      </div>

      {/* CSS for TOML syntax highlighting */}
      <style>{`
        .toml-comment { color: #6b7280; font-style: italic; }
        .toml-section { color: #f472b6; font-weight: 500; }
        .toml-key { color: #60a5fa; }
        .toml-string { color: #34d399; }
        .toml-number { color: #a78bfa; }
        .toml-boolean { color: #fbbf24; }
        .toml-datetime { color: #fb923c; }
        .toml-punctuation { color: #e2e8f0; }
      `}</style>
    </div>
  );
}
