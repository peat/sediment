require 'erb'
require 'nokogiri'

# sanity checking
if ARGV[0].nil? or ARGV[1].nil? or ARGV[2].nil?
  puts "svg-page.rb PATH_TO_SVG PATH_TO_OUTPUT TITLE"
  exit
end

template = ERB.new(File.read('template.html.erb'))

raw_svg = File.read(ARGV[0])
svg = Nokogiri::XML(raw_svg)

viewbox = svg.at_css("svg")["viewBox"].split(" ")
shape_count = svg.css("circle").count

output = template.result_with_hash(svg: raw_svg, shape_count: shape_count, title: ARGV[2])

File.write(ARGV[1], output)
