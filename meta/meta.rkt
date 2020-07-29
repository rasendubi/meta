#lang racket/base
(require racket/cmdline)
(require racket/stream)
(require racket/list)
(require racket/match)
(require json)

(require "meta/base.rkt")
(require "meta/core.rkt")
(require "meta/f.rkt")

(module+ main
  (define meta (make-parameter meta-empty))

  (define annotate-file (make-parameter #f))
  (define annotate-line (make-parameter #f))
  (define f-eval-id (make-parameter #f))

  (define meta-files (make-parameter '()))

  (command-line
   #:program "meta"
   #:once-any
   [("--annotate-file")
    "Annotate file"
    (annotate-file #t)]

   [("--annotate-line")
    al
    "Annotate line"
    (annotate-line al)]

   [("--f-eval")
    id
    "f-evaluate element with given id"
    (f-eval-id id)]

   #:multi
   [("-f" "--file") f
                    "Meta files to process"
                    (meta-files (cons f (meta-files)))])

  ;; (define (load-meta filename)
  ;;   (meta (meta-merge (meta) (read-meta-file filename))))

  (meta (apply meta-merge (meta) (map read-meta-file (meta-files))))

  (define (pretty-name id)
    (let ([identifiers (meta-lookup-identifiers (meta) id)])
      (if (empty? identifiers)
          (printf "(~a)" id)
          (printf "~a(~a)" (car identifiers) id))))

  (define (annotate-value attribute value)
    (let ([types (meta-lookup-value-types (meta) attribute)])
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

  (when (f-eval-id)
    (f-print (f-eval (meta) '() (f-eval-id)))))
