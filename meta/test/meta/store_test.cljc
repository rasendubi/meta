(ns meta.store-test
  (:require [meta.store :as store]
            [#?(:clj clojure.test :cljs cljs.test) :refer [deftest testing is]]))

(def ^:private test-store (store/datoms->store [["0" "0" "identifier"]
                                                ["0" "1" "2"]
                                                ["1" "1" "3"]
                                                ["1" "0" "Attribute.value-type"]
                                                ["2" "0" "String"]
                                                ["3" "0" "Reference"]
                                                ["4" "0" "comment"]
                                                ["0" "4" "Unique identifier of element"]
                                                ["0" "4" "Additional comment"]]))

(deftest store-test
  (testing "trivial constructor"
    (is (store/store? (store/datoms->store []))))

  (testing "get-entity-attribute"
    (is (= #{"identifier"} (store/eav test-store "0" "0"))))

  (testing "get-entity-attribute (multiple)"
    (is (= #{"Additional comment" "Unique identifier of element"}
           (store/eav test-store "0" "4"))))

  (testing "get-entity (attribute-value)"
    (is (= {"1" #{"3"}
            "0" #{"Attribute.value-type"}}
           (store/eav test-store "1"))))

  (testing "get-attribute-entity"
    (is (= #{"2"}
           (store/aev test-store "1" "0"))))

  (testing "get-attribute (entity-value)"
    (is (= {"0" #{"2"}
            "1" #{"3"}}
           (store/aev test-store "1"))))

  (testing "get-attribute-value"
    (is (= #{"0"} (store/ave test-store "0" "identifier"))))

  (testing "get-attribute (value-entity)"
    (is (= {"identifier" #{"0"}
            "Attribute.value-type" #{"1"}
            "String" #{"2"}
            "Reference" #{"3"}
            "comment" #{"4"}}
           (store/ave test-store "0")))))
