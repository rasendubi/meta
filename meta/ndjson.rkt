#lang racket/base
(require racket/stream)
(require json)

(module+ test
  (require rackunit))

(define (lines [in (current-input-port)])
  (let ([line (read-line in)])
    (if (eof-object? line)
        empty-stream
        (stream-cons line (lines in)))))

(define (string->jsexpr-maybe line)
  (let ([j (with-handlers ([exn:fail? (lambda (exn) #f)])
             (string->jsexpr line))])
    (and j (not (eof-object? j)) j)))

(module+ test
  (check-eq? (string->jsexpr-maybe "") #f)
  (check-eq? (string->jsexpr-maybe "blah") #f)
  (check-eq? (string->jsexpr-maybe "(~identifier(1000).blah = \"hello\")") #f))

(provide js-lines)
(define (js-lines [in (current-input-port)])
  (stream-filter (lambda (j)
                   (and j (not (eof-object? j))))
                 (stream-map string->jsexpr-maybe (lines))))
