require 'sinatra'
require 'json'

set :bind, '0.0.0.0'
set :port, 8081

get '/' do
  'Hello from Ruby with Superpilot!'
end

get '/health' do
  content_type :json
  { status: 'healthy' }.to_json
end
