# frozen_string_literal: true

Gem::Specification.new do |spec|
  spec.name          = "_vhp_theme"
  spec.version       = "1.0.0"
  spec.authors       = ["Leo Cavalcante"]
  spec.email         = ["leo@example.com"]

  spec.summary       = "A clean, modern GitHub Pages theme inspired by the PHP 8.5 landing page."
  spec.homepage      = "https://github.com/leocavalcante/vhp"
  spec.license       = "MIT"

  spec.files = `git ls-files -z`.split("\x0").select { |f|
    f.match(%r!^(assets|_layouts|_includes|_sass|LICENSE|README)!i)
  }

  spec.add_runtime_dependency "jekyll", "~> 4.2"
end
