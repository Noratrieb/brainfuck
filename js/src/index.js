import fs from 'fs/promises';

const RED = '\x1B[1;31m';
const RESET = '\x1B[1;0m';

function lex(string) {
  const tokens = [];

  for (let i = 0; i < string.length; i++) {
    const char = string[i];
    if (['+', '-', '>', '<', '.', ',', '[', ']'].includes(char)) {
      tokens.push({
        char,
        span: i
      });
    }
  }

  return tokens;
}

function ParseError(message, span) {
  this.message = message;
  this.span = span;
}

function Parser(tokens) {
  this.tokens = tokens;
  this.position = 0;
}

Parser.prototype.next = function() {
  const token = this.tokens[this.position];
  this.position++;
  return token;
};

Parser.prototype.parse = function(isLoop) {
  const body = [];
  let nextToken;
  while ((nextToken = this.next()) !== undefined) {
    switch (nextToken.char) {
      case '[': {
        const loopBody = this.parse(true);
        body.push(loopBody);
        break;
      }
      case ']': {
        if (isLoop) {
          return body;
        } else {
          throw new ParseError('No matching `[` found', nextToken.span);
        }
      }
      default: {
        body.push(nextToken);
      }
    }
  }

  if (isLoop) {
    throw new ParseError('No matching `]` found', this.tokens[this.tokens.length - 1].span);
  } else {
    return body;
  }
};

function reportError(source, message, span) {
  let lineIdx = 0;
  let lastNewlineIdx = 0;
  for (let i = 0; i < source.length; i++) {
    const char = source[i];
    if (i === span) {
      break;
    }
    if (char === '\n') {
      lineIdx++;
      lastNewlineIdx = i;
    }
  }

  const lines = source.split('\n');
  const line = lines[lineIdx];
  const lineNumber = String(lineIdx + 1);

  const linePrefix = `${lineNumber} | `;
  const lineSpan = span - lastNewlineIdx;

  console.error(`${RED}error: ${message}${RESET}`);
  console.error(`${linePrefix}${line}`);
  console.error(`${' '.repeat(linePrefix.length + lineSpan)}${RED}^${RESET}`);
}

const file = process.argv[2];

if (!file) {
  console.error('Usage: [filename]');
  process.exit(1);
}

const source = await fs.readFile(file, 'utf-8');
const tokens = lex(source);

const parser = new Parser(tokens);
try {
  const ast = parser.parse(false);
  console.log(ast);
} catch (parseError) {
  if (!(parseError instanceof ParseError)) {
    throw parseError;
  }
  reportError(source, parseError.message, parseError.span);
}
