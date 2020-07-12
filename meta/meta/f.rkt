#lang racket/base
(require "./base.rkt")
(require "./core.rkt")

(define StringLiteral "100")
(define StringLiteral.value "101")
(define RecordLiteral "102")
(define RecordLiteral.field "103")
(define RecordField "104")
(define RecordField.key "105")
(define RecordField.value "106")

(provide f-eval)
(define (f-eval meta id)
  (case (meta-lookup-types meta id)
    [(("100")) (eval-string meta id)]
    [(("102")) (eval-record meta id)]
    [(("104")) (eval-field meta id)]))

(define (eval-string meta id)
  (let ([v (vals (meta-lookup meta (list id StringLiteral.value #f)))])
    (car v)))

(define (eval-record meta id)
  (let ([fields (vals (meta-lookup meta (list id RecordLiteral.field #f)))])
    (make-immutable-hash (map (lambda (x) (eval-field meta x)) fields))))

(define (eval-field meta id)
  (let ([key (car (vals (meta-lookup meta (list id RecordField.key #f))))]
        [value (car (vals (meta-lookup meta (list id RecordField.value #f))))])
    (cons (f-eval meta key) (f-eval meta value))))

(provide f-print)
(define (f-print x)
  (println x))
