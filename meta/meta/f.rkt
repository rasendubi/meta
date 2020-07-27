#lang racket/base
(require pprint)

(require "./base.rkt")
(require "./core.rkt")

;; TODO: remove hard-code and find these by identifiers
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
(define Function "110")
(define Function.parameter "111")
(define Function.body "112")
(define FunctionParameter "113")
(define FunctionParameter.name "114")
(define Apply "115")
(define Apply.function "116")
(define Apply.argument "117")
(define Reference "118")
(define Reference.reference "119")

(define f-environment (make-parameter '()))

(define (f-environment-lookup id)
  (let ([ctx (findf (lambda (x) (hash-has-key? x id))
                    (f-environment))])
    (if ctx
        (hash-ref ctx id)
        (error 'f-context-lookup "Unable to find ~a, environment = ~v" id (f-environment)))))

(define-syntax-rule (with-environment env body ...)
  (parameterize ([f-environment (cons env (f-environment))])
    body ...))

(provide f-eval)
(define (f-eval meta id)
  (case (meta-lookup-types meta id)
    [(("100")) (eval-string meta id)]
    [(("102")) (eval-record meta id)]
    [(("104")) (eval-field meta id)]
    [(("107")) (eval-field-access meta id)]
    [(("110")) (eval-function meta id)]
    [(("115")) (eval-apply meta id)]
    [(("118")) (eval-reference meta id)]
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

(struct f-function (parameter-id body-id) #:transparent)

(define (eval-function meta id)
  (let ([parameter-id (meta-lookup-val meta id Function.parameter)]
        [body-id      (meta-lookup-val meta id Function.body)])
    (f-function parameter-id body-id)))

(define (eval-apply meta id)
  (let ([function-id (meta-lookup-val meta id Apply.function)]
        [argument-id (meta-lookup-val meta id Apply.argument)])
    (f-apply meta (f-eval meta function-id) (f-eval meta argument-id))))

(define (f-apply meta function argument)
  (with-environment (make-immutable-hash (list (cons (f-function-parameter-id function) argument)))
    (f-eval meta (f-function-body-id function))))

(define (eval-reference meta id)
  (let ([reference-id (meta-lookup-val meta id Reference.reference)])
    (f-environment-lookup reference-id)))

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
    [(("110")) (pretty-function meta id)]
    [(("113")) (pretty-function-parameter meta id)]
    [(("115")) (pretty-apply meta id)]
    [(("118")) (pretty-reference meta id)]
    [else (h-append (text "(") (text id) (text ")"))]
    ;; [else (error 'f-pretty "Unable to pretty-print ~a" id)]
    ))

(define (pretty-string meta id)
  (let ([val (meta-lookup-val meta id StringLiteral.value)])
    (text (format "~v" val))))

(define (pretty-record meta id)
  (let ([field-ids (meta-lookup-vals meta id RecordLiteral.field)])
    (group
     (h-append
      (text "{")
      (nest 2 (h-append
               line
               (h-concat
                (apply-infix (h-append comma line) (map (lambda (x) (f-pretty meta x)) field-ids)))))
      line
      (text "}")))))

(define (pretty-field meta id)
  (let ([key-id (meta-lookup-val meta id RecordField.key)]
        [value-id (meta-lookup-val meta id RecordField.value)])
    (group (nest 2 (h-append
                    (group (h-append (text "[") (f-pretty meta key-id) (text "]") (text ":")))

                    line
                    (f-pretty meta value-id))))))

(define (pretty-field-access meta id)
  (let ([record-id (meta-lookup-val meta id FieldAccess.record)]
        [field-id  (meta-lookup-val meta id FieldAccess.field)])
    (group (nest 2
                 (h-append
                  (f-pretty meta record-id)
                  break
                  dot
                  (text "[")
                  (f-pretty meta field-id)
                  (text "]"))))))

(define (pretty-function meta id)
  (let ([parameter-id (meta-lookup-val meta id Function.parameter)]
        [body-id      (meta-lookup-val meta id Function.body)])
    (group (nest 2 (h-append
                    (text "\\")
                    (f-pretty meta parameter-id)
                    (text " ")
                    (text "->")
                    line
                    (f-pretty meta body-id))))))

(define (pretty-function-parameter meta id)
  (let ([name (meta-lookup-val meta id FunctionParameter.name)])
    (text name)))

(define (pretty-apply meta id)
  (let ([function-id (meta-lookup-val meta id Apply.function)]
        [argument-id (meta-lookup-val meta id Apply.argument)])
    (group (nest 2 (h-append
                    (text "(")
                    (f-pretty meta function-id)
                    (text ")")
                    line
                    (text "(")
                    (f-pretty meta argument-id)
                    (text ")"))))))

(define (pretty-reference meta id)
  (let ([reference-id (meta-lookup-val meta id Reference.reference)])
    (f-pretty meta reference-id)))
