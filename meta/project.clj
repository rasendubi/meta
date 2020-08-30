(defproject meta "0.0.1-SNAPSHOT"
  :description "Data pretending to be code"
  :url "https://github.com/rasendubi/meta"
  :dependencies [[org.clojure/clojure "1.10.1"]
                 [org.clojure/clojurescript "1.10.758"]
                 [com.bhauman/figwheel-main "0.2.11"]
                 [com.bhauman/rebel-readline-cljs "0.1.4"]
                 [cheshire "5.10.0"]
                 [reagent "1.0.0-alpha2"]]
  :resource-paths ["target" "resources"]
  :aliases {"fig"       ["trampoline" "run" "-m" "figwheel.main"]
            "build-dev" ["trampoline" "run" "-m" "figwheel.main" "-b" "dev" "-r"]
            "cljs-test" ["trampoline" "run" "-m" "figwheel.main" "-co" "tests.cljs.edn" "-m" "meta.test-runner"]}
  :main ^:skip-aot meta.main
  :target-path "target/%s"
  :clean-targets ^{:protect false} [:target-path :compile-path])
