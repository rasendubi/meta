(ns meta.layout.core-test
  #?(:cljs (:require-macros [meta.layout.test-utils]))
  (:require [meta.layout.core :as l]
            #?(:clj [meta.layout.test-utils])
            #?(:clj  [clojure.test :refer [deftest testing is]]
               :cljs [cljs.test :refer [deftest testing is]])))

(defn- text [s]
  (l/cell (count s) s))

(deftest layout-test
  (testing "empty"
    (is (layout= (l/empty))))

  (testing "cell"
    (is (layout= (text "hello")
                 (text "hello"))))

  (testing "concat cells"
    (is (layout= (l/concat (list (text "hello") (text " ") (text "world!")))
                 (text "hello") (text " ") (text "world!"))))

  (testing "concat concat"
    (is (layout= (l/concat (list (l/concat (list (text "1") (text "2")))
                                 (l/concat (list (text "3") (text "4")))))
                 (text "1") (text "2") (text "3") (text "4"))))

  (testing "line"
    (is (layout= (l/concat (list (text "hello")
                                 (l/line)
                                 (text "world!")))
                 (text "hello")
                 (merge (l/line) {:width 0})
                 (merge (l/indent) {:width 0})
                 (text "world!"))))

  (testing "line + nest"
    (is (layout=
         (l/concat (list (text "hello") (l/nest 2 (l/concat (list (l/line) (text "world!"))))))
         (text "hello") (merge (l/line) {:width 0}) (merge (l/indent) {:width 2}) (text "world!"))))

  (testing "group"
    (testing "text"
      (is (layout= (l/group (text "blah"))
                   (text "blah"))))

    (testing "line"
      (is (layout= (l/group (l/line))
                   (l/empty))))

    (testing "line (alt)"
      (is (layout= (l/group (l/line (text "alt")))
                   (text "alt"))))

    (testing "empty"
      (is (layout= (l/group (l/empty)))))

    (testing "concat + flat line"
      (is (layout= (l/group (l/concat (list (text "text")
                                            (l/line (text " "))
                                            (text "more text"))))
                   (text "text") (text " ") (text "more text"))))

    (testing "concat + break line"
      (is (layout= (l/group (l/concat (list (text "long text")
                                            (l/line (text " "))
                                            (text "more long text"))))
                   (text "long text")
                   (merge (l/line (text " ")) {:width 0})
                   (merge (l/indent) {:width 0})
                   (text "more long text"))))))
