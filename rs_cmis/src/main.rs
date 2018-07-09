extern crate hyper;
extern crate rpassword;
extern crate ansi_term;
extern crate serde;
extern crate serde_json;

use hyper::Client;
use std::io::{Read, Write, stdout};
use std::borrow::Borrow;
use std::cell::RefCell;
use ansi_term::Colour::{Yellow, Red, Blue};


#[derive(Debug)]
struct CMISShell {
    repo: Repository,
    repo_pwd: String,
    repo_dir_stack: RefCell<Vec<String>>,
    repo_pwd_info: RefCell<Vec<serde_json::Value>>,
    repo_info: serde_json::Value,
    repo_root_dir_url: String,
    repo_url: String,
}

impl CMISShell {
    pub fn new() -> CMISShell {

        let repo = Repository::new(getInput(RepoProperty::Url),
                                   getInput(RepoProperty::Name),
                                   getInput(RepoProperty::User),
                                   getInput(RepoProperty::Pass));

        let repo = Repository::new(String::from("http://localhost:8080/chemistry-opencmis-server-inmemory-1.1.0/browser",),
                                   String::from("A1"),
                                   String::from("Admim"),
                                   String::from("admim"));
        let repo_data: serde_json::Value = serde_json::from_str(&(repo.get_repo_info()[..]))
            .unwrap();
        let mut repo_pwd_info_response =
            repo.call_url(&(repo_data
                                .get(&repo.name)
                                .unwrap()
                                .get("rootFolderUrl")
                                .unwrap()
                                .as_str()
                                .unwrap()));
        let mut repo_pwd_info = String::new();
        repo_pwd_info_response.read_to_string(&mut repo_pwd_info);
        let repo_pwd_info:serde_json::Value = serde_json::from_str(&(repo_pwd_info)[..]).unwrap();

        CMISShell {
            repo_url: repo_data
                .get(&repo.name)
                .unwrap()
                .get("repositoryUrl")
                .unwrap()
                .as_str()
                .unwrap()
                .to_owned(),
            repo_root_dir_url: repo_data
                .get(&repo.name)
                .unwrap()
                .get("rootFolderUrl")
                .unwrap()
                .as_str()
                .unwrap()
                .to_owned(),
            repo_info: repo_data,
            repo: repo,
            repo_pwd: String::from("/"),
            repo_dir_stack:RefCell::new(vec![String::from("root")]),
            repo_pwd_info: RefCell::new(vec![repo_pwd_info]),
        }
    }


    pub fn ls(&self) {
        
        for file in (&self.repo_pwd_info).borrow().last().unwrap().get("objects").unwrap().as_array().unwrap().iter() {
            let file_type = file.get("object")
                .unwrap()
                .get("properties")
                .unwrap()
                .get("cmis:objectTypeId")
                .unwrap()
                .get("value")
                .unwrap()
                .as_str()
                .unwrap();
            let file_name = file.get("object")
                .unwrap()
                .get("properties")
                .unwrap()
                .get("cmis:name")
                .unwrap()
                .get("value")
                .unwrap()
                .as_str()
                .unwrap();
            match file_type {
                "ComplexType" => println!("{}", file_name),
                "cmis:folder" => println!("{}/", ansi_term::Colour::Blue.paint(file_name)),
                _ => {
                    println!("{}     Type:{}",
                             ansi_term::Colour::Yellow.paint(file_name),
                             ansi_term::Colour::Yellow.paint(file_type))
                }
            }
        }
    }

    pub fn pwd(&self) -> String{
        self.repo_dir_stack.borrow().join("/").clone()
    }

    fn is_file(&self)-> bool{
        true
    }
    // pub fn cd(&mut self, target_dir: &str) {
        // let mut new_path = String::new();
        // new_path.push_str(&self.repo_url);
        // new_path.push_str("/root");
        // new_path.push_str(&self.repo_pwd[..]);
        // new_path.push_str(target_dir);
        // let mut response = self.repo.call_url(&new_path[..]);
        // let mut response_string = String::new();
        // response.read_to_string(&mut response_string);
        // self.repo_pwd_info = serde_json::from_str(&response_string[..]).unwrap();
        // self.repo_pwd.push_str(target_dir);
        // println!("{:?}", self.repo_pwd);
    // }


    pub fn cd(&self,target_dir:&str) {
        match target_dir {
            ".." => {
            self.repo_pwd_info.borrow_mut().pop();
            self.repo_dir_stack.borrow_mut().pop();
            ()
            },
            "." => {
            
        let mut folder_url = (&self).repo_url.clone();
        folder_url.push_str("/");
        folder_url.push_str(&self.repo_dir_stack.borrow().join("/"));
        
        let mut response = self.repo.call_url(&folder_url[..]);
        let mut response_string = String::new();
        response.read_to_string(&mut response_string);
            self.repo_pwd_info.borrow_mut().pop();
        self.repo_pwd_info.borrow_mut().push(serde_json::from_str(&response_string[..]).unwrap());
        &self.ls();
        ()
            },
            _ => {
        let mut folder_url = (&self).repo_url.clone();
        folder_url.push_str("/");
        folder_url.push_str(&self.repo_dir_stack.borrow().join("/"));
        folder_url.push_str("/");
        folder_url.push_str(target_dir);
        let mut response = self.repo.call_url(&folder_url[..]);
        let mut response_string = String::new();
        response.read_to_string(&mut response_string);
        self.repo_pwd_info.borrow_mut().push(serde_json::from_str(&response_string[..]).unwrap());
        self.repo_dir_stack.borrow_mut().push(target_dir.to_owned());
        ()
            }
        }
    }
    
    pub fn cat(&self,target_file:&str){
        let mut file_url = (&self).repo_url.clone();
        file_url.push_str("/");
        file_url.push_str(&self.pwd()[..]);
        file_url.push_str("/");
        file_url.push_str(target_file);
        let mut response = self.repo.call_url(&file_url[..]);
        let mut response_string = String::new();
        response.read_to_string(&mut response_string);
        println!("{}",response_string);
    }

    pub fn prompt(&self) {
        print!("/{} >", &self.pwd());
        std::io::stdout().flush();
    }

}

enum RepoProperty {
    Url,
    Name,
    User,
    Pass,
}

#[derive(Debug)]
struct Repository {
    url: String,
    name: String,
    username: String,
    password: String,
}


impl Repository {
    pub fn new(url: String, name: String, username: String, password: String) -> Repository {
        Repository {
            url: url,
            name: name,
            username: username,
            password: password,
        }
    }

    pub fn get_basic_auth(&self) -> hyper::header::Basic {
        hyper::header::Basic {
            username: (&self.username).to_owned(),
            password: Some((&self.password).to_owned()),
        }
    }


    pub fn get_repo_info(&self) -> String {
        let mut response = self.call_url(&self.url);
        let mut response_string = String::new();
        &response.read_to_string(&mut response_string);
        response_string
    }

    fn call_url(&self, url: &str) -> hyper::client::Response {
        let client = hyper::Client::new();
        let mut header = hyper::header::Headers::new();
        header.set(hyper::header::Authorization((&self).get_basic_auth()));
        let mut res = client.get(url).headers(header).send().unwrap();
        match res.status {
            hyper::status::StatusCode::Ok => (),//println!("Connected to Repository: {}", &self.name),
            _ => panic!("Failed to connect to repoUrl:{}", url),
        };
        res
    }
}

fn main() {
    println!("{}",
             Red.bold().paint("**** Welcome to a rusty CMIShell ****"));
    // println!("{}", Red.paint("***** Author: Harikrishnan Menon *****"));
    println!("{}",
             Red.bold().paint("****   Version: 0.0.1-pre-Alpha  ****"));
    println!("{}",
             Red.bold().paint("*************************************"));
    println!("");

    let mut shell = CMISShell::new();
    loop {
        let mut input = String::new();
        shell.prompt();
        std::io::stdin()
            .read_line(&mut input)
            .expect("Failed to read input");
        let mut input = input.trim();
        let mut input_iter:std::str::SplitWhitespace = input.split_whitespace();
        // println!("{:?}",input);
        match input_iter.next() {
            None => continue,
            Some(cmd) => { 
                match cmd {
                    "ls" => shell.ls(),
                    "pwd" => println!("/{}", shell.pwd()),
                    "cd" => {
                        match input_iter.next() {
                            None => {shell.pwd();()},
                            Some(arg)=>shell.cd(arg),
                        }
                    },
                    "cat" => {
                        match input_iter.next() {
                            None =>println!("Invalid FileName"),
                            Some(arg)=>shell.cat(arg),
                        }
                    }
                    "" => continue,
                    "print" => println!("{:?}",shell.repo_dir_stack.get_mut()),
                    "exit" => std::process::exit(1),
                    _ => println!("Invalid command"),
            }
            },
        }
    }

}

fn getInput(property: RepoProperty) -> String {
    match property {
        RepoProperty::Url => print!("Enter the Repository Url: "),
        RepoProperty::Name => print!("Enter the Repository Name: "),
        RepoProperty::User => print!("Enter the Repository UserName: "),
        RepoProperty::Pass => print!("Enter the Repository Password: "),
    }
    std::io::stdout().flush();
    let mut input = String::new();
    match property {
        RepoProperty::Pass => {
            input = rpassword::read_password().unwrap();
            ()
        }
        _ => {
            std::io::stdin()
                .read_line(&mut input)
                .expect("Failed to read input");
            ()
        }
    }
    let input = input.trim();
    input.to_owned()
}
