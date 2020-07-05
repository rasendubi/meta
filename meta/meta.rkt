#lang racket/base
(require racket/cmdline)
(require racket/stream)
(require racket/list)
(require racket/match)
(require json)

(require "ndjson.rkt")

(require "meta/base.rkt")

(module+ main
  (define meta (make-parameter '()))

  (define annotate-file (make-parameter #f))
  (define annotate-line (make-parameter #f))

  (define meta-files (make-parameter '()))

  (command-line
   #:program "meta"
   #:once-any
   [("--annotate-file") "Annotate file"
                        (annotate-file #t)]
   [("--annotate-line") al
                        "Annotate line"
                        (annotate-line al)]
   #:multi
   [("-f" "--file") f
                    "Meta files to process"
                    (meta-files (cons f (meta-files)))])

  (define (load-meta filename)
    (meta (append* (meta) (list (read-meta-file filename)))))

  (meta (append* (meta) (map read-meta-file (meta-files))))

  (define (meta-lookup pattern)
    (define (match-two expected actual)
      (or (not expected)
          (equal? expected actual)))

    (define (filter-f x)
      (andmap match-two pattern x))

    (filter filter-f (meta)))

  (define (vals xs)
    (map (lambda (x) (list-ref x 2)) xs))

  (define (meta-lookup-identifiers id)
    (vals (meta-lookup (list id "0" #f))))

  (define (meta-lookup-value-types attr-id)
    (vals (meta-lookup (list attr-id "1" #f))))

  (define (pretty-name id)
    (let ([identifiers (meta-lookup-identifiers id)])
      (if (empty? identifiers)
          (printf "(~a)" id)
          (printf "~a(~a)" (car identifiers) id))))

  (define (annotate-value attribute value)
    (let ([types (meta-lookup-value-types attribute)])
      (if (equal? '("3") types) ; Reference
          (pretty-name value)
          (write value))))

  (define (annotate atom)
    (match-let ([(list entity attribute value) atom])
      (pretty-name entity)
      (display ".")
      (pretty-name attribute)
      (display " = ")
      (annotate-value attribute value)
      (newline)
      ))

  (when (annotate-file)
    (for ([atom (meta)])
        (annotate atom)))

  (when (annotate-line)
    (let ([j (string->jsexpr (annotate-line))])
      (annotate j)))
  )
