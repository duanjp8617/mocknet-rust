import requests
import json
from argparse import ArgumentParser
from os import system


MOCKNET_SERVER_DIR='/home/pengyang/repo/mocknet-rust/target/debug'
INDRADB_SERVER_DIR='/home/pengyang/repo/indradb/target/debug'
IP="172.21.103.147"
MOCKNETPORT='3030'
INDRADBPORT='3031'
SERVERCHECKPORT='4040'
IMAGE="ubuntu:20.10"
CONFIG_FILE='/home/pengyang/repo/mocknet-rust/target/debug/cluster_config_template.json'

POST_HEADER={
            "Content-Type": "application/json",
            "Accept": "*/*",
            "Accept-Encoding": "gzip, deflate, br",
            "Connection": "keep-alive"
        }

# parser that deal with initial commands
Parser_init = ArgumentParser(description='Process the initial command')
Parser_init.add_argument(
    '-msd', '--mocknet_dir', action='store', type = str, dest='mocknet_dir',
    default=MOCKNET_SERVER_DIR,
    help='give the direction of mocknet-rust files, default=/home/pengyang/repo/mocknet-rust/target/debug'
)
Parser_init.add_argument(
    '-isd', '--indradb_dir', action='store', type = str, dest='indradb_dir',
    default=INDRADB_SERVER_DIR,
    help='the direction of indradb files, default=/home/pengyang/repo/indradb/target/debug'
)
Parser_init.add_argument(
    '-ip', action='store', type= str, dest='ip_address',
    default=IP,
    help='the ipv4 address of mocknet servers, dafault=172.21.103.147'
)
Parser_init.add_argument(
    '-mnp', '--mocknet_port', action='store', type = str, dest='mocknet_port',
    default=MOCKNETPORT,
    help='the port that mocknet server use, defalut=3030'
)
Parser_init.add_argument(
    '-idp', '--indradb_port', action='store', type = str, dest='indradb_port',
    default=INDRADBPORT,
    help='the port that indradb server use, default=3031'
)
Parser_init.add_argument(
    '-scp', '--server_check_port', action='store', type = str, dest='server_check_port',
    default=SERVERCHECKPORT,
    help='the port that server check using, default=4040'
)
Parser_init.add_argument(
    '-img', '--image', action='store', type = str, dest='image',
    default=IMAGE,
    help='the version of your operation system, default=ubuntu:20.10'
)
Parser_init.add_argument(
    '-c', '--config', action = 'store', type = str, dest = 'config_file',
    default=CONFIG_FILE,
    help = 'set the topology file address, default=/home/pengyang/repo/mocknet-rust/target/debug/cluster_config_template.json'
)

# launch indradb
'''system(
    'sudo docker run ' +  
    '--net=host ' +
    '--privileged ' +
    '-v /tmp:/tmp '+
    '-v ' + args.indradb_dir + ':/workspace ' +
    '--name ' + indradb_server_name +
    ' -it ' +
    '-d ' + args.image +
    ' /workspace/'+indradb_server_name + ' -a '+ args.ip_address + ':' + args.indradb_port +
    ' rocksdb /tmp/mocknet')

# launch mocknet
system(
    'sudo docker run ' +
    '--net=host ' +
    '--privileged ' +
    '-v ' + args.mocknet_dir + ':/workspace ' +
    '--name ' + mocknet_server_name +
    ' -it ' +
    '-d ' + args.image + 
    ' /workspace/' + 'mocknet_server' +
    ' --wrap-addr ' + args.ip_address + ':' + args.mocknet_port +
    ' --indradb-addr ' + args.ip_address + ':' + args.indradb_port +
    ' --cluster-config ' + args.config_file
)

# launch server_check
system(
    'sudo docker run ' +
    '--net=host ' +
    '--privileged ' +
    '-v ' + args.mocknet_dir + ':/workspace ' +
    '--name server-check ' +
    ' -it ' +
    '-d ' + args.image +
    ' /workspace/server_check ' +
    ' --wrap-addr ' + args.ip_address + ':' + args.server_check_port
)'''

# the stage of initializing server
print("*** note: input '-h' or '--help' to show these argument needed for initializing server, or 'exit' to quit")
print("*** note: ending input with no argument will set these as default value, it's recommended to comfirm them by 'help'")
while True:
    character = input("mocknet(initializing)> ")
    if character == "exit":
        break
    command_init = character.split()
    try:
        args_init = Parser_init.parse_args(command_init)
    except:
        print("")
    else:
        break

# present run server on local
system("gnome-terminal -- '" + args_init.indradb_dir + '/indradb-server' + "'")
system("gnome-terminal -- '" + args_init.mocknet_dir + '/api_mockserver' + "'")
system("gnome-terminal -- " + args_init.mocknet_dir + '/mocknet_server' + 
       ' --cluster-config ' + args_init.config_file
       ) 
print("*** successfully initialize the mocknet server")

# parser that deal with operation commands
Parser_op = ArgumentParser(description='Process the operation command')

subparsers = Parser_op.add_subparsers(description='[SUBDOMMAND] + \'-h\' to get it\'s help', dest='CmdType')
# subparser for initializing emunet
parser_emuinit = subparsers.add_parser('netinit', help='initialize a emunet')
parser_emuinit.add_argument(
    '--uuid', action = 'store', type = str, dest = 'emunet_uuid', required = True,
    help = 'the uuid of emunet that you want initialize'
)
parser_emuinit.add_argument(
    '--nodes', action = 'store', type = str, dest = 'nodes', required = True,
    nargs = '+', metavar = 'ID, DESCRIPTION',
    help = 'the nodes information of emunet that you want initialize'
)
parser_emuinit.add_argument(
    '--links', action = 'store', type = str, dest = 'links', required = True,
    nargs = '+', metavar = 'ID1, ID2, DESCRIPTION',
    help = 'the links information of emunet that you want initialize'
)

# subparser for register user
parser_register = subparsers.add_parser('register', help = 'register as a user')
parser_register.add_argument(
    '-n', '--user_name', action = 'store', type = str, required = True,
    default = None,
    help = 'the name you want to register in the server'
)

# subparser for create emunet
parser_create = subparsers.add_parser('netcrt', help = 'create a emunet under a user with specified capacity')
parser_create.add_argument(
    '-u', '--user', action = 'store', type = str, dest = 'user_name', default = None,
    required = True, help = 'the user name that you want to create emunet for'
)
parser_create.add_argument(
    '-n', '--net', action = 'store', type = str, dest = 'net_name', default = None,
    required = True, help = 'the emunet name that you want to create'
)
parser_create.add_argument(
    '-c', '--capacity', action = 'store', type = int, dest = 'capacity', default = None,
    required = True, help = 'the emunet capacity that you want to create'
)

# subparser for get emunet infomation
parser_emuinfo = subparsers.add_parser('listall', help = 'get the global information')

# subparser for get emunet list
parser_emulist = subparsers.add_parser('netlist', help = 'get emunet list of a user')
parser_emulist.add_argument(
    '-u', '--user', action = 'store', type = str, default = None, required = True,
    dest = 'user_name',
    help = 'the user name that you want to list out'
)

# subparser for delete emunet
parser_emudel = subparsers.add_parser('netdel', help = 'delete a emunet')
parser_emudel.add_argument(
    '-u', '--uuid', action = 'store', type = str, default = None, required = True,
    dest = 'uuid',
    help = 'the uuid of emunet that you want to delete'
)

# subparser for delete user
parser_usrdel = subparsers.add_parser('usrdel', help = 'delete a user')
parser_usrdel.add_argument(
    '-n', '--name', action = 'store', type = str, default = None, required = True,
    help = 'the user name you want to delete'
)

# subparser for update emunet
parser_netupdate = subparsers.add_parser('netupdate', help='update a emunet\'s nodes and links')
parser_netupdate.add_argument(
    '--uuid', action = 'store', type = str, dest = 'emunet_uuid', required = True,
    help = 'the uuid of emunet that you want update'
)
parser_netupdate.add_argument(
    '--nodes', action = 'store', type = str, dest = 'nodes', required = True,
    nargs = '+', metavar = 'ID, DESCRIPTION',
    help = 'the nodes information of emunet that you want update'
)
parser_netupdate.add_argument(
    '--links', action = 'store', type = str, dest = 'links', required = True,
    nargs = '+', metavar = 'ID1, ID2, DESCRIPTION',
    help = 'the links information of emunet that you want update'
)

# subparser for get specified emunet information
parser_netinfo = subparsers.add_parser('netinfo', help = 'get information of a emunet')
parser_netinfo.add_argument(
    '-u', '--uuid', action = 'store', type = str, default = None, required = True, 
    help = 'the emunet uuid you want to delete'
)

# subparser for get specified emunet state
parser_netstate = subparsers.add_parser('netstat', help = 'get state of a emunet')
parser_netstate.add_argument(
    '-u', '--uuid', action = 'store', type = str, default = None, required = True, 
    help = 'the emunet uuid you want to get state for'
)

# opreation functions
def register_user(url, name):
    body = {
        "name": name
    }
    response = requests.post(
        url = url,
        data = json.dumps(body),
        headers = POST_HEADER
    )
    return response

def create_emunet(url, user_name, emunet_name, emunet_capacity):
    body = {
        "user": user_name,
        "emunet": emunet_name,
        "capacity": emunet_capacity
    }
    response = requests.post(
        url = url,
        data = json.dumps(body),
        headers = POST_HEADER
    )
    return response

def init_emunet(url, info):
    response = requests.post(
        url = url,
        data = json.dumps(info),
        headers = POST_HEADER
    )
    return response

def get_net_info(url):
    response = requests.post(
        url = url,
        data = None,
        headers = POST_HEADER
    )
    return response

def emunet_list(url, name):
    body = {
        "user": name
    }
    response = requests.post(
        url = url,
        data = json.dumps(body),
        headers = POST_HEADER
    )
    return response

def delete_user(url, name):
    body = {
        'name': name
    }
    response = requests.post(
        url = url,
        data = json.dumps(body),
        headers = POST_HEADER
    )
    return response

def get_emunet_info(url, uuid):
    body = {
        'emunet_uuid': uuid
    }
    response = requests.post(
        url = url,
        data = json.dumps(body),
        headers = POST_HEADER
    )
    return response

# the stage of normal operation 
print("*** note: input '-h' or '--help' to show these subcommands for operation, or 'exit' to quit")
while True:
    character = input("mocknet> ")
    if character == "exit":
        break
    command_op = character.split() 
    try:
        args_op = Parser_op.parse_args(command_op)
        if args_op.CmdType == 'register':
            response = register_user(url="http://localhost:3030/v1/register_user", name = args_op.user_name)
            response_json = response.json()
            #print(response.status_code)
            if response_json['success'] == True:
                print("successfully register as '%s'" % args_op.user_name)
            else:
                print("error! the message is:%s" % response_json['message'])
        
        if args_op.CmdType == 'netcrt':
            response = create_emunet(url="http://localhost:3030/v1/create_emunet", 
                        user_name = args_op.user_name,
                        emunet_name = args_op.net_name,
                        emunet_capacity = args_op.capacity,
            )
            #print(response.text)
            #print(response.status_code)
            response_json = response.json()
            if response_json['success'] == True:
                print("successfully create emunet named '%s' with user '%s' and capacity '%s', the UUID of emunet is: '%s'" 
                % (args_op.user_name, args_op.net_name, args_op.capacity, response_json['data']))
            else:
                print("error! the message is: %s" % response_json['message'])
        
        if args_op.CmdType == 'netinit':
            nodes_dicts = list()
            links_dicts = list()
            for i in range(0, len(args_op.nodes)//2):
                nodes_dicts.append(dict(id=int(args_op.nodes[i*2]), description=args_op.nodes[i*2+1]))
            for i in range(0, len(args_op.links)//3):
                links_dicts.append(dict(edge_id=[int(args_op.links[i*3]), int(args_op.links[i*3+1])], description=args_op.links[i*3+2]))
            total_dict = dict(emunet_uuid=args_op.emunet_uuid, devs=nodes_dicts, links=links_dicts)
            response = init_emunet(url='http://localhost:3030/v1/init_emunet', info=total_dict)
            #print(response.text)
            #print(response.status_code)
            response_json = response.json()
            if response_json['success'] == True:
                print('successfullt init the emunet, now it\'s working!')
            else:
                print("error! the message is: %s" % response_json['message'])

        if args_op.CmdType == 'listall':
            response = get_net_info(url='http://localhost:3030/v1/list_all')
            response_json = response.json()
            response_data = response_json['data']
            fmt_data = json.dumps(response_data, sort_keys=True, indent=4,separators=(',',':'))
            if response_json['success'] == True:
                print(fmt_data)
            else:
                print("error! the message is: %s" % response_json['message'])

        if args_op.CmdType == 'netlist':
            response = emunet_list(url='http://localhost:3030/v1/list_emunet', name=args_op.user_name)
            response_json = response.json()
            if response_json['success'] == True:
                for net in response_json['data']:
                    print('name: %s, uuid: %s' % (net, response_json['data'][net]))
            else:
                print("error! the message is: %s" % response_json['message'])

        if args_op.CmdType == 'netdel':
            response = delete_user(url='http://localhost:3030/v1/delete_emunet', name=args_op.name)
            response_json = response.json()
            if response_json['success'] == True:
                print('successfully delete user: %s', args_op.name)
            else:
                print("error! the message is: %s" % response_json['message'])

        if args_op.CmdType == 'netupdate':
            nodes_dicts = list()
            links_dicts = list()
            for i in range(0, len(args_op.nodes)//2):
                nodes_dicts.append(dict(id=int(args_op.nodes[i*2]), description=args_op.nodes[i*2+1]))
            for i in range(0, len(args_op.links)//3):
                links_dicts.append(dict(edge_id=[int(args_op.links[i*3]), int(args_op.links[i*3+1])], description=args_op.links[i*3+2]))
            total_dict = dict(emunet_uuid=args_op.emunet_uuid, devs=nodes_dicts, links=links_dicts)
            # same operation function with init_emunet()
            response = init_emunet(url='http://localhost:3030/v1/update_emunet', info=total_dict)
            response_json = response.json()
            if response_json['success'] == True:
                print('successfullt update the emunet, now it\'s working!')
            else:
                print("error! the message is: %s" % response_json['message'])

        if args_op.CmdType == 'usrdel':
            response = get_net_info(url='http://localhost:3030/v1/delete_user', uuid=args_op.uuid)
            response_json = response.json()
            if response_json['success'] == True:
                print("suscessfully delete the emunet!")
            else:
                print("error! the message is: %s" % response_json['message'])

        if args_op.CmdType == 'netinfo':
            response = get_emunet_info(url='http://localhost:3030/v1/get_emunet_info', uuid=args_op.uuid)
            response_json = response.json()
            response_data = response_json['data']
            fmt_data = json.dumps(response_data, sort_keys=True, indent=4,separators=(',',':'))
            if response_json['success'] == True:
                print(fmt_data)
            else:
                print("error! the message is: %s" % response_json['message'])

        if args_op.CmdType == 'netstat':
            response = get_emunet_info(url='http://localhost:3030/v1/get_emunet_state', uuid=args_op.uuid)
            response_json = response.json()
            response_data = response_json['data']
            fmt_data = json.dumps(response_data, sort_keys=True, indent=4,separators=(',',':'))
            if response_json['success'] == True:
                print(fmt_data)
            else:
                print("error! the message is: %s" % response_json['message'])

    except:
        if character != '--help' and character != '' :
            print("")

    

