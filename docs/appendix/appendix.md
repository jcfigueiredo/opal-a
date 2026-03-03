# Appendix

## A. Links

- Keyboard and symbols
  - [Help pages Mac OS keyboard](https://forlang.wsu.edu/help-pages/help-pages-keyboards-os-x/)
  - [Special symbols](https://discussions.apple.com/thread/6535997?start=0&tstart=0)
- Generic language
  - [Algebraic data types: without them, we all suffer](https://genericlanguage.wordpress.com/2015/06/09/algebraic-data-types-without-them-we-all-suffer/)
  - [Advice on writing a programming language](https://genericlanguage.wordpress.com/2014/02/04/advice-on-writing-a-programming-language/)
  - [Programmers as glue factories](https://genericlanguage.wordpress.com/2014/03/29/programmers-as-glue-factories/)
  - [More on abstraction](https://genericlanguage.wordpress.com/2015/01/08/more-on-abstraction/)
- [ANTLR](http://www.antlr.org/)
- [ATS language](http://www.ats-lang.org/)
- Tokenizer -> [Lexical scanning in Go](https://blog.golang.org/two-go-talks-lexical-scanning-in-go-and)
- Parser -> [Top Down Operator Precedence](http://javascript.crockford.com/tdop/tdop.html)
- [Akka / actor model / concurrency](http://readwrite.com/2014/07/10/akka-jonas-boner-concurrency-distributed-computing-internet-of-things/)
  - [pyutil](https://github.com/zooko/pyutil)
  - [ANTLR4 Python target](https://github.com/antlr/antlr4/blob/master/doc/python-target.md)
- Ruby [BNF](https://www.cse.buffalo.edu//~regan/cse305/RubyBNF.pdf)

## B. Topics

- BNF [1](https://en.wikipedia.org/wiki/Backus%E2%80%93Naur_Form), [2](https://en.wikipedia.org/wiki/Extended_Backus%E2%80%93Naur_Form), [3](https://en.wikipedia.org/wiki/Augmented_Backus%E2%80%93Naur_Form)
- Type inference in [Crystal](http://crystal-lang.org/2013/09/23/type-inference-part-1.html)
- [ANTLR Ruby grammar](https://github.com/antlr/grammars-v4/tree/master/ruby)
- [Memetalk bits.py](https://github.com/thiago-silva/memetalk/blob/master/sugarfoot/pyutils/bits.py)
- [Almost Y combinator in JavaScript](http://blog.klipse.tech/lambda/2016/08/10/almost-y-combinator-javascript.html)
- [Set theory](https://en.wikipedia.org/wiki/Set_theory)

## C. Tutorials

- [How to write an interpreter?](https://www.youtube.com/watch?v=1h1mM7VwNGo)
- [Let's Build a Simple Interpreter](https://ruslanspivak.com/lsbasi-part1/)

## D. References

- [Mini](https://github.com/keleshev/mini)
- [Memetalk](https://github.com/thiago-silva/memetalk)
- Crystal
  - [kernel.cr](https://github.com/crystal-lang/crystal/blob/master/src/kernel.cr)
- Pixie
  - [target.py](https://github.com/pixie-lang/pixie/blob/master/target.py)
- Kermit
  - [Kermit interpreter](https://bitbucket.org/pypy/example-interpreter/src/a00d0f9c36f151112d35708b82035a541fe6f16f/kermit/?at=default)
- [RPython/RPLY interpreter](http://joshsharp.com.au/blog/rpython-rply-interpreter-1.html)
  - [Braid](https://github.com/joshsharp/braid)
- Cycy
  - [Cycy parser](https://github.com/Magnetic/cycy/blob/master/cycy/parser/core.py)
- Bytecode
  - [lang-rio](https://libraries.io/github/edcrypt/lang-rio)

## E. Ideas

- Go's [GoVet / GoFmt](https://golang.org/cmd/gofmt/) — built-in linting and formatting
- Optional type system (e.g., method signatures)
- Use of Examples to define arguments, maybe classes or entities
- Guard clauses (also using Examples)
- Gather data of usage to suggest optimization
- Intelligent Assistant to help during development time (providing Examples, or based on historical data)
- Runtime modes
  - **Rock** — Simple, single-box application, local memory, local disk
  - **Rough** — Halfway between Rock and Polished
  - **Polished** — Production-ready, cluster, shared memory, virtual networked storage
- Internal Supervisor/Systemd support
- Pattern matching like [Rust patterns](https://doc.rust-lang.org/book/second-edition/ch18-03-pattern-syntax.html)
- Syntax references
  - If — [Crystal suffix syntax](http://crystal-lang.org/docs/syntax_and_semantics/as_a_suffix.html)
  - Range — [Crystal range literals](http://crystal-lang.org/docs/syntax_and_semantics/literals/range.html)
- Unicode references — [Unicode table](http://unicode-table.com/en/#telugu)
- Syntax highlighter (cobalt) — [markup.su](http://markup.su/highlighter/)
