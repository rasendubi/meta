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
(define Identifier "113")
(define Identifier.name "114")
(define Apply "115")
(define Apply.function "116")
(define Apply.argument "117")
(define Reference "118")
(define Reference.reference "119")
(define Letrec "120")
(define Letrec.binding "121")
(define Letrec.value "122")
(define Binding "123")
(define Binding.identifier "124")
(define Binding.value "125")

(define (env-lookup env id)
  (let ([ctx (findf (lambda (x) (hash-has-key? x id)) env)])
    (if ctx
        (hash-ref ctx id)
        (error 'f-context-lookup "Unable to find ~a, environment = ~v" id env))))

(define (new-scope scope env)
  (cons scope env))

;; TODO: rework as expr + env, instead of taking in lambda
(struct f-thunk (evaluated data) #:transparent #:mutable)
(define (f-thunk-evaluate th)
  (when (not (f-thunk-evaluated th))
    (let* ([evaluator (f-thunk-data th)]
           [result (evaluator)])
      (set-f-thunk-data! th
                         (if (f-thunk? result)
                             (f-thunk-force result)
                             result))
      (set-f-thunk-evaluated! th #t))))
(define (f-thunk-force th)
  (when (not (f-thunk-evaluated th))
    (f-thunk-evaluate th))
  (f-thunk-data th))
(define (f-thunk-force-deep th)
  (let ([v (f-thunk-force th)])
    (cond
      [(hash? v) (make-immutable-hash (hash-map v (lambda (key value)
                                                    (cons key (f-thunk-force-deep value)))))]
      [else v])))
;; TODO: detect recursive evaluation

(define-syntax-rule (f-thunk-new body ...)
  (f-thunk #f (lambda () body ...)))

(define (eval-string meta env id)
  (f-thunk-new (meta-lookup-val meta id StringLiteral.value)))

(define (eval-record meta env id)
  (let ([fields (meta-lookup-vals meta id RecordLiteral.field)])
    (f-thunk-new
     (make-immutable-hash (map (lambda (x) (eval-field meta env x)) fields)))))

(define (eval-field meta env id)
  (let ([key-id (meta-lookup-val meta id RecordField.key)]
        [value-id (meta-lookup-val meta id RecordField.value)])
    (cons (f-thunk-force (f-eval meta env key-id)) (f-eval meta env value-id))))

(define (eval-field-access meta env id)
  (let* ([record-id (meta-lookup-val meta id FieldAccess.record)]
         [field-id  (meta-lookup-val meta id FieldAccess.field)]
         [record    (f-eval meta env record-id)]
         [field     (f-eval meta env field-id)])
    (f-thunk-new
     (hash-ref (f-thunk-force record) (f-thunk-force field)))))

(struct f-function (parameter-id body-id env) #:transparent)

(define (eval-function meta env id)
  (let ([parameter-id (meta-lookup-val meta id Function.parameter)]
        [body-id      (meta-lookup-val meta id Function.body)])
    (f-thunk-new
     (f-function parameter-id body-id env))))

(define (eval-apply meta env id)
  (let ([function-id (meta-lookup-val meta id Apply.function)]
        [argument-id (meta-lookup-val meta id Apply.argument)])
    (f-thunk-new
     (f-apply meta env
              (f-thunk-force (f-eval meta env function-id))
              (f-eval meta env argument-id)))))

(define (f-apply meta env function argument)
  (let* ([function-env (f-function-env function)]
         [new-env (new-scope (make-immutable-hash (list (cons (f-function-parameter-id function) argument))) function-env)])
    (f-eval meta new-env (f-function-body-id function))))

(define (eval-reference meta env id)
  (let ([reference-id (meta-lookup-val meta id Reference.reference)])
    (f-thunk-new
     (env-lookup env reference-id))))

(define (eval-letrec meta env id)
  (define scope (make-hash '()))
  (define new-env (new-scope scope env))

  (let* ([binding-ids (meta-lookup-vals meta id Letrec.binding)]
         [bindings (map (lambda (x) (eval-binding meta new-env x)) binding-ids)]
         [value-id (meta-lookup-val meta id Letrec.value)])
    (for/list ([b bindings])
      (hash-set! scope (car b) (cdr b)))

    (f-thunk-new
     (f-eval meta new-env value-id))))

(define (eval-binding meta env id)
  (let ([identifier-id (meta-lookup-val meta id Binding.identifier)]
        [value-id (meta-lookup-val meta id Binding.value)])
    (cons identifier-id (f-eval meta env value-id))))

(provide f-eval)
(define (f-eval meta env id)
  (define evaluators
    (make-immutable-hash
     `((,StringLiteral . ,eval-string)
       (,RecordLiteral . ,eval-record)
       (,FieldAccess   . ,eval-field-access)
       (,Function      . ,eval-function)
       (,Apply         . ,eval-apply)
       (,Reference     . ,eval-reference)
       (,Letrec        . ,eval-letrec))))

  (let ([types (meta-lookup-types meta id)])
    (if (null? types)
        (error 'f-eval "~a does not have a type" id)
        (let* ([type (car types)]
               [evaluator (hash-ref evaluators type #f)])
          (if evaluator
              (evaluator meta env id)
              (error 'f-eval "No evaluator for type ~a" type))))))

(provide f-print)
(define (f-print x)
  (define (f-format-string s)
    (text (format "~s" s)))

  (define (f-format-record x)
    (group (h-append
            (text "{")
            (nest 2 (h-append
                     line
                     (v-concat (map (lambda (pair)
                                      (group
                                       (h-append
                                        (f-format (car pair))
                                        (text " ")
                                        (text "=")
                                        (nest 2
                                              (h-append
                                               line
                                               (f-format-thunk (cdr pair))
                                               (text ";"))))))
                                    (hash->list x)))
                     ))
            line
            (text "}"))))

  (define (f-format x)
    (cond
      [(string? x)
       (f-format-string x)]
      [(hash? x)
       (f-format-record x)]
      [else
       (error 'f-print "Don't know how to print ~v" x)]))

  (define (f-format-thunk x)
    (f-format (f-thunk-force x)))

  (pretty-print (f-format-thunk x)))


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

(define (pretty-identifier meta id)
  (let ([name (meta-lookup-val meta id Identifier.name)])
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

(define (pretty-letrec meta id)
  (let ([binding-ids (meta-lookup-vals meta id Letrec.binding)]
        [value-id (meta-lookup-val meta id Letrec.value)])
    (group (h-append
            (text "let")
            (text " ")
            (text "{")
            (h-append
             (nest 2
                   (h-append
                    line
                    (v-concat (map (lambda (binding-id) (h-append (f-pretty meta binding-id) (text ";"))) binding-ids))))
             line)
            (text "}")
            (text " ")
            (text "in")
            (group (nest 2 (h-append
                            line
                            (f-pretty meta value-id))))))))

(define (pretty-binding meta id)
  (let ([identifier-id (meta-lookup-val meta id Binding.identifier)]
        [value-id (meta-lookup-val meta id Binding.value)])
    (group (nest 2 (h-append
                    (f-pretty meta identifier-id)
                    (text " ")
                    (text "=")
                    line
                    (f-pretty meta value-id))))))

(provide f-pretty)
(define (f-pretty meta id)
  (define formatters
    (make-immutable-hash
     `((,StringLiteral     . ,pretty-string)
       (,RecordLiteral     . ,pretty-record)
       (,RecordField       . ,pretty-field)
       (,FieldAccess       . ,pretty-field-access)
       (,Function          . ,pretty-function)
       (,Identifier        . ,pretty-identifier)
       (,Apply             . ,pretty-apply)
       (,Reference         . ,pretty-reference)
       (,Letrec            . ,pretty-letrec)
       (,Binding           . ,pretty-binding))))

  (let ([types (meta-lookup-types meta id)])
    (if (null? types)
        (error 'f-pretty "~a does not have a type" id)
        (let* ([type (car types)]
               [formatter (hash-ref formatters type #f)])
          (if formatter
              (formatter meta id)
              (h-append (text "(") (text id) (text ")")))))))
