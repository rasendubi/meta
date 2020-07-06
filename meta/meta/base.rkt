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

(define (vals xs)
  (map (lambda (x) (list-ref x 2)) xs))
