#lang racket/base
(require racket/stream)

(require "../ndjson.rkt")

(define (parse-meta-file [in (current-input-port)])
  (stream->list (js-lines in)))

(provide read-meta-file)
(define (read-meta-file filename)
  (with-input-from-file filename parse-meta-file
    #:mode 'text))
