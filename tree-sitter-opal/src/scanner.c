#include "tree_sitter/parser.h"
#include <string.h>
#include <stdlib.h>

enum TokenType {
  FSTRING_START_DOUBLE,
  FSTRING_START_SINGLE,
  FSTRING_CONTENT,
  FSTRING_END,
  INTERPOLATION_START,
  INTERPOLATION_END,
  MULTILINE_COMMENT,
};

typedef struct {
  int brace_depth;
  char quote_char; // '"' or '\'' or 0
} Scanner;

void *tree_sitter_opal_external_scanner_create(void) {
  Scanner *scanner = calloc(1, sizeof(Scanner));
  return scanner;
}

void tree_sitter_opal_external_scanner_destroy(void *payload) {
  free(payload);
}

unsigned tree_sitter_opal_external_scanner_serialize(void *payload, char *buffer) {
  Scanner *scanner = (Scanner *)payload;
  buffer[0] = (char)scanner->brace_depth;
  buffer[1] = scanner->quote_char;
  return 2;
}

void tree_sitter_opal_external_scanner_deserialize(void *payload, const char *buffer, unsigned length) {
  Scanner *scanner = (Scanner *)payload;
  if (length >= 2) {
    scanner->brace_depth = (int)buffer[0];
    scanner->quote_char = buffer[1];
  } else {
    scanner->brace_depth = 0;
    scanner->quote_char = 0;
  }
}

static void advance(TSLexer *lexer) {
  lexer->advance(lexer, false);
}

bool tree_sitter_opal_external_scanner_scan(void *payload, TSLexer *lexer, const bool *valid_symbols) {
  Scanner *scanner = (Scanner *)payload;

  // Inside f-string: handle interpolation end, start, content, or string end
  if (scanner->quote_char != 0) {
    // Interpolation end: }
    if (valid_symbols[INTERPOLATION_END] && lexer->lookahead == '}') {
      advance(lexer);
      lexer->result_symbol = INTERPOLATION_END;
      return true;
    }

    // Interpolation start: {
    if (valid_symbols[INTERPOLATION_START] && lexer->lookahead == '{') {
      advance(lexer);
      lexer->result_symbol = INTERPOLATION_START;
      return true;
    }

    // F-string end: matching quote
    if (valid_symbols[FSTRING_END] && lexer->lookahead == (unsigned)scanner->quote_char) {
      advance(lexer);
      scanner->quote_char = 0;
      lexer->result_symbol = FSTRING_END;
      return true;
    }

    // F-string content: everything else until { or quote or EOF
    if (valid_symbols[FSTRING_CONTENT]) {
      bool has_content = false;
      while (lexer->lookahead != 0 &&
             lexer->lookahead != '{' &&
             lexer->lookahead != (unsigned)scanner->quote_char) {
        if (lexer->lookahead == '\\') {
          advance(lexer); // skip backslash
          if (lexer->lookahead != 0) advance(lexer); // skip escaped char
        } else {
          advance(lexer);
        }
        has_content = true;
      }
      if (has_content) {
        lexer->result_symbol = FSTRING_CONTENT;
        return true;
      }
    }

    return false;
  }

  // Skip whitespace
  while (lexer->lookahead == ' ' || lexer->lookahead == '\t' || lexer->lookahead == '\r' || lexer->lookahead == '\n') {
    lexer->advance(lexer, true);
  }

  // Check for multiline comment: ### ... ###
  if (valid_symbols[MULTILINE_COMMENT] && lexer->lookahead == '#') {
    lexer->mark_end(lexer);
    advance(lexer);
    if (lexer->lookahead == '#') {
      advance(lexer);
      if (lexer->lookahead == '#') {
        advance(lexer);
        // Now consume until we find ###
        int hash_count = 0;
        while (lexer->lookahead != 0) {
          if (lexer->lookahead == '#') {
            hash_count++;
            if (hash_count == 3) {
              advance(lexer);
              lexer->mark_end(lexer);
              lexer->result_symbol = MULTILINE_COMMENT;
              return true;
            }
          } else {
            hash_count = 0;
          }
          advance(lexer);
        }
      }
    }
    return false;
  }

  // F-string start: f" or f'
  if (valid_symbols[FSTRING_START_DOUBLE] && lexer->lookahead == 'f') {
    lexer->mark_end(lexer);
    advance(lexer);
    if (lexer->lookahead == '"') {
      advance(lexer);
      scanner->quote_char = '"';
      scanner->brace_depth = 0;
      lexer->mark_end(lexer);
      lexer->result_symbol = FSTRING_START_DOUBLE;
      return true;
    }
    if (lexer->lookahead == '\'') {
      advance(lexer);
      scanner->quote_char = '\'';
      scanner->brace_depth = 0;
      lexer->mark_end(lexer);
      lexer->result_symbol = FSTRING_START_SINGLE;
      return true;
    }
    return false;
  }

  return false;
}
