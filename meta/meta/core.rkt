#lang racket/base
(require racket/contract)

(require "base.rkt")

(provide
 (contract-out
  [meta-lookup-identifiers (-> meta? string? (listof string?))]
  [meta-lookup-value-types (-> meta? string? (listof string?))]
  [meta-lookup-types (-> meta? string? (listof string?))]))

(define identifier "0")

;; TODO: un-hardcode attribute/value-type
(define attribute/value-type "1")
(define type "5")

(define (meta-lookup-identifiers meta id)
  (vals (meta-lookup meta (list id identifier #f))))

(define (meta-lookup-value-types meta attr-id)
  (vals (meta-lookup meta (list attr-id attribute/value-type #f))))

(define (meta-lookup-types meta id)
  (vals (meta-lookup meta (list id type #f))))
