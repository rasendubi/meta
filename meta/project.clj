(defproject meta "0.0.1-SNAPSHOT"
  :description "Data pretending to be code"
  :url "https://github.com/rasendubi/meta"
  :dependencies [[org.clojure/clojure "1.10.1"]
                 [org.clojure/clojurescript "1.10.520"]
                 [com.bhauman/figwheel-main "0.2.11"]
                 [com.bhauman/rebel-readline-cljs "0.1.4"]
                 [datascript "1.0.0"]
                 [cheshire "5.10.0"]]
  :resource-paths ["target" "resources"]
  :aliases {"fig"       ["trampoline" "run" "-m" "figwheel.main"]
            "build-dev" ["trampoline" "run" "-m" "figwheel.main" "-b" "dev" "-r"]}
  :main ^:skip-aot meta.main
  :target-path "target/%s/")
