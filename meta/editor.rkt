#lang racket/base
(require racket/cmdline)
(require pprint)

(require "meta/base.rkt")
(require "meta/f.rkt")

(module+ main
  (define meta (make-parameter meta-empty))
  (define pretty (make-parameter #f))

  (command-line
   #:program "meta/editor"
   #:once-any
   [("--pretty")
    id
    "Pretty-print element."
    (pretty id)]
   #:args files
   (meta (apply meta-merge (meta) (map read-meta-file files))))

  (when (pretty)
    (parameterize ([current-page-width 80])
      (pretty-print (f-pretty (meta) (pretty))))))
