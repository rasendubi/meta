#lang racket/base
(require pprint)

(require "./base.rkt")
(require "./core.rkt")

(define StringLiteral "100")
(define StringLiteral.value "101")
(define RecordLiteral "102")
(define RecordLiteral.field "103")
(define RecordField "104")
(define RecordField.key "105")
(define RecordField.value "106")
(define FieldAccess "107")
(define FieldAccess.record "108")
(define FieldAccess.field "109")

(provide f-eval)
(define (f-eval meta id)
  (case (meta-lookup-types meta id)
    [(("100")) (eval-string meta id)]
    [(("102")) (eval-record meta id)]
    [(("104")) (eval-field meta id)]
    [(("107")) (eval-field-access meta id)]
    [else (error 'f-eval "Unable to evaluate ~a" id)]))

(define (eval-string meta id)
  (meta-lookup-val meta id StringLiteral.value))

(define (eval-record meta id)
  (let ([fields (meta-lookup-vals meta id RecordLiteral.field)])
    (make-immutable-hash (map (lambda (x) (f-eval meta x)) fields))))

(define (eval-field meta id)
  (let ([key-id (meta-lookup-val meta id RecordField.key)]
        [value-id (meta-lookup-val meta id RecordField.value)])
    (cons (f-eval meta key-id) (f-eval meta value-id))))

(define (eval-field-access meta id)
  (let* ([record-id (meta-lookup-val meta id FieldAccess.record)]
         [field-id  (meta-lookup-val meta id FieldAccess.field)]
         [record    (f-eval meta record-id)]
         [field     (f-eval meta field-id)])
    (hash-ref record field)))

(provide f-print)
(define (f-print x)
  (println x))


(provide f-pretty)
(define (f-pretty meta id)
  (case (meta-lookup-types meta id)
    [(("100")) (pretty-string meta id)]
    [(("102")) (pretty-record meta id)]
    [(("104")) (pretty-field meta id)]
    [(("107")) (pretty-field-access meta id)]
    [else (error 'f-pretty "Unable to pretty-print ~a" id)]))

(define (pretty-string meta id)
  (let ([val (meta-lookup-val meta id StringLiteral.value)])
    (text (format "~v" val))))

(define (pretty-record meta id)
  (let ([field-ids (meta-lookup-vals meta id RecordLiteral.field)])
    (group (h-append (text "{")
                     (nest 2 (h-append
                              line
                              (h-concat (apply-infix (h-append comma line)
                                                     (map (lambda (x) (f-pretty meta x)) field-ids)))))
                     line
                     (text "}")))))

(define (pretty-field meta id)
  (let ([key-id (meta-lookup-val meta id RecordField.key)]
        [value-id (meta-lookup-val meta id RecordField.value)])
    (group
     (h-append
      (group (h-append (text "[") (f-pretty meta key-id)   (text "]") (text ":")))
      (nest 2
            (h-append
             line
             (f-pretty meta value-id)))))))

(define (pretty-field-access meta id)
  (let ([record-id (meta-lookup-val meta id FieldAccess.record)]
        [field-id  (meta-lookup-val meta id FieldAccess.field)])
    (group (h-append
            (f-pretty meta record-id)
            (nest 2
                  (h-append
                   break
                   dot
                   (text "[")
                   (f-pretty meta field-id)
                   (text "]")))))))
