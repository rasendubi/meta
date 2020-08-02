#lang racket/base
(require racket/stream)
(require racket/contract)
(require racket/list)

(require "../ndjson.rkt")

(define atom? list?)

(provide meta?)
(define meta? (listof atom?))

(provide
 (contract-out
  [meta-empty meta?]
  [read-meta-file (-> string? meta?)]
  [meta-merge (->* () () #:rest (listof meta?) meta?)]
  [meta-lookup (-> meta? any/c (listof atom?))]
  [meta-lookup-vals (-> meta? any/c any/c (listof any/c))]
  [meta-lookup-val (-> meta? any/c any/c any/c)]
  [vals (-> (listof atom?) (listof string?))]))

(define meta-empty '())

(define (parse-meta-file [in (current-input-port)])
  (stream->list (js-lines in)))

(define (read-meta-file filename)
  (with-input-from-file filename parse-meta-file
    #:mode 'text))

(define (meta-merge . meta)
  (apply append meta))

(define (meta-lookup meta pattern)
  (define (match-two expected actual)
    (or (not expected)
        (equal? expected actual)))

  (define (filter-f x)
    (andmap match-two pattern x))

  (filter filter-f meta))

(define (meta-lookup-vals meta entity attr)
  (vals (meta-lookup meta (list entity attr #f))))

(define (meta-lookup-val meta entity attr)
  (car (meta-lookup-vals meta entity attr)))

(define (vals xs)
  (map (lambda (x) (list-ref x 2)) xs))
