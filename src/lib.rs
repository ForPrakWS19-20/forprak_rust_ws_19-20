#![allow(unused)]
use std::fs::{File, OpenOptions, Metadata, metadata};
use std::io::{SeekFrom, Seek, Read, Write};
use std::{fs, mem};
use std::iter::Map;
use std::collections::HashMap;
use std::time::SystemTime;


// BFA, Block file access, bietet die Moeglichkeit, Block zu get und put
// Ein Block hat eine einzige ID, richtet nach einem Bereich von xxByte nach xxByte in File
// Gebe bestimmte ID, kriege den Block, kriege den Teil v  cxdsaq   c   on File

pub struct BFA{
    pub block_size: usize,
    pub file: File,
    //metadata sollte auf Typ Map sein
    metadaten: HashMap<String,String>,
    //1 for true, 0 for false
    update_file: Vec<bool>,
    reserved_file: Vec<bool>,
    reserve_count:usize
}

pub struct Block{
    pub contents: Vec<u8>,
}

pub struct RTree{
    root_id:usize,
    bfa: BFA,
    dimension:usize,
    total_id: usize,
    M: usize
}



enum Node{
    //mittels von der id aus inneren Knoten das Blatt(als Block) ausholen
    //dann zum Blatt serialisieren
    Leaf{
        content: Vec<LeafElement>
    },
    InnerNode{
        content: Vec<InnerElement>
    },
}


pub struct LeafElement {
    daten:Vec<Point>,
    mbr:MBRect
}

pub struct InnerElement {
    //MBRect von Kinder
    mbrs:Vec<MBRect>,
    //ID, mit welcher Blöcke vom BFA geholt werden können
    children:usize
}


pub struct Point{
    //Pos 0: x, Pos 1: y
    //coor:Vec<f64>
    x:f64,
    y:f64
}

pub struct MBRect{
    id: usize,
    botton_left:Point,
    top_right:Point,
}

impl Node{

    pub fn from_block(block: &mut Block) -> Self {
        let node = bincode::deserialize(block.contents.as_slice());
        node.expect("error")
    }

    pub fn get_innernode_content(&mut self) -> Option<&mut Vec<InnerElement>> {
        match self {
            Node::InnerNode {content} => {
                Some(content)
            }
            Node::Leaf {content} => {
                None
            }
        }
    }

    pub fn get_leaf_content(&mut self) -> Option<&mut Vec<LeafElement>> {
        match self {
            Node::Leaf {content} => {
                Some(content)
            }
            Node::InnerNode {content} => {
                None
            }
        }
    }

    pub fn set_innernode_content(&mut self, new_content: Vec<InnerElement>){
        match self {
            Node::InnerNode {content} => {
                *content = new_content
            }
            Node::Leaf {content} => {}
        }
    }

    pub fn set_leaf_content(&mut self, new_content: Vec<LeafElement>){
        match self {
            Node::Leaf {content} => {
                *content = new_content
            }
            Node::InnerNode {content} => {}
        }
    }

    pub fn serialize(&self) -> Vec<u8> {
        bincode::serialize(&self).unwrap()
    }

    pub fn to_block (&self) -> Block {
        let obj = self.serialize();
        let block = Block::new(obj);
        block
    }
}

impl InnerElement{
    pub fn new(id:usize, mbr: Vec<MBRect>,children: usize) -> Self {
        InnerElement(id,mbr,children)
    }
}

impl LeafElement{
    pub fn new(daten: Vec<Point>,mbr: MBRect) -> Self {
        LeafElement(daten,mbr)
    }
}

impl Point{
    pub fn new(x:f64, y:f64) -> Self{
        Point(x,y)
    }
}

impl MBRect{
    pub fn new(bl:Point, tp:Point, id:usize) -> Self{
        MBRect(id,bl,tp)
    }

    fn mbr_of(p1:Point, p2:Point, id:usize) -> MBRect{
        let minx1 = r1.botton_left.x;
        let miny1 = r1.botton_left.y;
        let maxx1 = r1.top_right.x;
        let maxy1 = r1.top_right.y;
        let minx2 = r2.botton_left.x;
        let miny2 = r2.botton_left.y;
        let maxx2 = r2.top_right.x;
        let maxy2 = r2.top_right.y;
        let minx = min(minx1,minx2);
        let miny = min(miny1,miny2);
        let maxx = max(maxx1,maxx2);
        let maxy = max(maxy1,maxy2);
        let min= Point::new(minx,miny);
        let max= Point::new(maxx,maxy);
        let rect= MBRect::new(min,max,id);
        rect
    }

    fn mbr_of_rects(r1:&MBRect, r2:&MBRect, id:usize) -> MBRect{
        let minx1 = r1.botton_left.x;
        let miny1 = r1.botton_left.y;
        let maxx1 = r1.top_right.x;
        let maxy1 = r1.top_right.y;
        let minx2 = r2.botton_left.x;
        let miny2 = r2.botton_left.y;
        let maxx2 = r2.top_right.x;
        let maxy2 = r2.top_right.y;
        let minx = min(minx1,minx2);
        let miny = min(miny1,miny2);
        let maxx = max(maxx1,maxx2);
        let maxy = max(maxy1,maxy2);
        let min= Point::new(minx,miny);
        let max= Point::new(maxx,maxy);
        let rect: MBRect = MBRect::new(min,max,id);
        rect
    }

    fn overlap(&self, other: &MBRect) -> bool {
        let minx1 = self.botton_left.x;
        let miny1 = self.botton_left.y;
        let maxx1 = self.top_right.x;
        let maxy1 = self.top_right.y;
        let minx2 = other.botton_left.x;
        let miny2 = other.botton_left.y;
        let maxx2 = other.top_right.x;
        let maxy2 = other.top_right.y;
        let minx = max(minx1,minx2);
        let miny = max(miny1,miny2);
        let maxx = min(maxx1,maxx2);
        let maxy = min(maxy1,maxy2);
        return (minx < maxx) && (miny < maxy);
    }

    fn rect_area (& self) -> f64 {
        let minx = self.botton_left.x;
        let miny = self.botton_left.y;
        let maxx = self.top_right.x;
        let maxy = self.top_right.y;
        let area = (maxy - miny) * (maxx - minx);
        return area;
    }
}


impl RTree{
    fn new(mut bfa: BFA, total_id: usize, M: usize) -> Self{
        let root_id = bfa.get_root();
        let dimension: usize = 2;

        RTree{root_id, bfa, dimension, total_id, M}
    }

    pub fn node_is_leaf(&mut self, node: &Node) -> bool{
        match node {
            Node::Leaf => true,
            Node::InnerNode => false,
        }
    }

//////////////////////////////////////////////////////////////////////////////////////////
    //Basis Funktion
    fn get_node(&mut self, id: usize) -> Node {
        let mut block = self.bfa.get(id);
        let node = Node::from_block(& mut block);
    }

    fn get_leaf_rect(&mut self, id: usize) -> Option<Vec<MBRect>> {
        let node = self.get_node(id);
        match node {
            Node::Leaf {content} => {
                let mut res: Vec<MBRect> = Vec::new();
                for i in content {
                    res.push(i.mbr);
                }
                Some(res)
            }
            Node::InnerNode {content} => {
                None
            }
        }
    }

    fn get_innernode_rect(&mut self, id: usize) -> Option<Vec<Vec<MBRect>>>{
        let node = self.get_node(id);
        match node {
            Node::Leaf {content} => {
                None
            }
            Node::InnerNode {content} => {
                let mut res: Vec<Vec<MBRect>> = Vec::new();
                for i in content {
                    res.push(i.mbrs);
                }
                Some(res)
            }
        }
    }

    //get rect area with given id
    fn rect_area_id(&mut self, id: usize) -> f64 {
        let rect = self.get_rect(id);
        let area = rect.rect_area();
        area
    }
/////////////////////////////////////////////////////////////////////////////////////////

    fn search_overlap_innernode(&mut self, rect: &MBRect, tmp: usize, mut overlapped: Vec<usize>) -> Option<Vec<usize>>{
        let mut tmp_node = self.get_node(tmp);
        match tmp_node {
            Node::Leaf { content } => { None }
            Node::InnerNode { content } => {
                for i in tmp_node.get_innernode_content().unwrap() {
                    let mut status = false;
                    for j in i.mbrs {
                        if rect.overlap(&j) {
                            let children_node = self.get_node(i.children);
                            match children_node {
                                Node::Leaf { content } => { overlapped.push(i.children) }
                                Node::InnerNode { content } => {
                                    return self.search_overlap_innernode(rect, i.children, overlapped);
                                }
                            }
                        }
                    }
                }
                if overlapped.len() != 0 {
                    Some(overlapped)
                }
                else { None } // when status = false, nothing adds to overlapped, return none instead of Some(empty vec)
            }
        }
    }

    fn search_overlap_leafnode(&mut self, rect: &MBRect, tmp: usize) -> Option<Vec<Point>>{
        let mut tmp_node = self.get_node(tmp);
        let mut res: Vec<Point> = Vec::new();
        match tmp_node {
            Node::InnerNode {content} => {None}
            Node::Leaf {content} => {
                for i in tmp_node.get_leaf_content().unwrap() {
                    if rect.overlap(&i.mbr) {
                        for j in i.daten {
                            res.push(j);
                        }
                    }
                }
                Some(res)
            }
        }
        None
    }

    fn search(&mut self, rect: &MBRect) -> Option<Vec<Point>>{
        //生产根Block，Node和Rect
        //Erstelle Block, Node von der Wurzel
        let root_id = self.root_id;
        let root_node = self.get_node(root_id);
        match root_node {
            Node::Leaf {content} => {
                let res = self.search_overlap_leafnode(rect,root_id);
                res
            }
            //Node ist InnerNode
            Node::InnerNode {content} => {
                let overlapped = self.search_overlap_innernode(rect,root_id,Vec::new()).unwrap();
                let mut res: Vec<Point> = Vec::new();
                for i in overlapped {
                    let point = self.search_overlap_leafnode(rect,i).unwrap();
                    for j in point {
                        res.push(j);
                    }
                }
                /*let mut tmp = root_id;
                let mut tmp_node = self.get_node(tmp);
                //search tmp node children, if they with rect overlap
                //next: overlapped children of tmp node
                let mut next = self.search_overlap_innernode(rect,tmp);
                //tmp node is InnerNode
                while !self.node_is_leaf(&tmp_node) {
                    //Da alle Kinderbäume stehen in gleicher Etage, das heißt, wenn der erste Kinderbaum ein Blatt ist, sind alle.
                    tmp = *next.get(0).unwrap();
                    tmp_node = self.get_node(tmp);
                    let mut next_next = self.search_overlap_innernode(rect,tmp);
                    //Lege die neue bekommene Enkelkinderbäume am Ende des Vectors von Kinderbäume
                    next.append(&mut next_next);
                    //Entferne den erste Kinderbaum und gehe immer weiter
                    next.remove(0);

                }
                let mut res: Vec<Point> = Vec::new();
                for i in next {
                    //直至子树集中第一个子树为叶子 应该只剩一个
                    //overlap_children里应该储存通过search_overlap_gruppe得到的id
                    let mut data = self.search_overlap_leafnode(rect,i);
                    //TODO match data is Some/None, bin nicht sicher
                    match data {
                        None => (),
                        Some(i) => i
                    }
                    res.append(&mut data.unwrap());
                }
                Some(res)*/
            }
        }
    }




    ////////////////////////////////////////////////////////////////////////////////
    //insert a new node into RTree

    fn insert(&mut self, insert_rect: MBRect){
        let pos = self.choose_leaf(self.root_id,&insert_rect);
        let mut chosen_block = self.bfa.get(pos);
        let chosen_node = Node::from_block(&mut chosen_block);
        let child_num = chosen_node.children().unwrap().len();

        if child_num < self.M {
            let new_id = self.total_id + 1;
            chosen_node.children().unwrap().push(new_id);
            //TODO was in content eig?
            //let content =
            //TODO///////////////
            let new_node_element = LeafElement::new(vec![],mbr);
            let mut new_elem = Vec::new();
            new_elem.push(new_node_element);
            let new_node = Node::Leaf { content: new_elem};
            let new_block = Node::to_block(& new_node);
            self.bfa.update(new_id,new_block);
        }
        else {
            //TODO hinzufügen fkt hier oder in split
            self.split(tmp_id, Vec::new(), Vec::new());
            //involke AdjustTree if split was performed
            self.adjust_tree(tmp_id);
        }
    }



    //Hilfsfunktion fuer choose_leaf und choose_leaves
    fn add_area(&mut self, small_rect: &MBRect, big_rect: &MBRect) -> f64{
        let sbl = &small_rect.botton_left;
        let str = &small_rect.top_right;
        let bbl = &big_rect.botton_left;
        let btr = &big_rect.top_right;
        let area =  (btr.y - bbl.y) * (btr.x - bbl.x);
        let mut add_area: f64 = 0 as f64;
        if sbl >= bbl && str <= btr{ }
        else {
            let neu_bl_x = min(sbl.x,bbl.x);
            let neu_bl_y = min(sbl.y,bbl.y);
            let neu_tr_x = max(str.x,btr.x);
            let neu_tr_y = max(str.y,btr.y);
            let neu_area = (neu_tr_y - neu_bl_y) * (neu_tr_x - neu_bl_x);
            add_area = neu_area - area;
        }
        add_area
    }




    //TODO FERTIG
    //Hilfsfunktion fuer insert
    //Find position for new record
    fn choose_leaf(&mut self, index: usize, insert_rect:&MBRect) -> usize {
        let mut tmp_block = self.bfa.get(index);
        let tmp_node = Node::from_block(& mut tmp_block);
        let tmp_rect = tmp_node.rect();
        let child_id = tmp_node.children().unwrap()[0];
        let mut child_block = self.bfa.get(child_id);
        let child_node = Node::from_block(& mut child_block);

        //until child node is not a leaf
        //choose this node and then can add new leaf under this node
        while !self.node_is_leaf(& child_node) {
            let mut area = self.rect_area_id(index);
            let mut id = tmp_node.children().unwrap()[0];
            for i in 0..tmp_node.children().len-1 {
                let tmp = tmp_node.children().unwrap()[i];
                let add = self.add_area(insert_rect, &tmp_rect);
                if add <= area {
                    area = add;
                    id = tmp;
                }
            }
            self.choose_leaf(id, insert_rect);
        }
        index
    }


    //acsend from a leaf node with id to the root
    // adjusting covering rectangles
    fn adjust_tree(& mut self, id: usize){
        let mut block = self.bfa.get(id);
        let node = Node::from_block(& mut block);
        //check if done， stop
        if id == self.root_id{

        } else {//adjust covering rectangle in parent entry
            //wie bekommt man parent node?
            //TODO/////
            //Beim Einfügen: die Funktion rekursiv aufrufen (bis zum Blatt)
            //beim Zurücklaufen der Rekursion hat automatisch den Parent
            //ODER
            //Liste von Knoten IDs mitführen und diese dann hinterher reparieren

        }

    }

    //add a new entry to a full node with M children
    //divide the collection of M+1 entries between two nodes
    //linear cost algorithm
    //在M+1个entries中选择2个作为两个新组的第一个elem
    //如何选择这两个entries？
    //选择浪费最大面积的一组 如果他们被放在了相同的组那最棒了
    //覆盖两个entries的矩形面积减去他们各自的矩形的面积
    //剩余entries将按次分配到一组
    //每次都会计算将剩余entries添加到组所需的面积扩展
    //分配的entry展示了两组间最大的差

    //采用思想:
    //直接加入节点 再判断是否超过M个子集 进行split&adjust
    //
    fn split_new(&mut self, id: usize){
        let m = max(2 as usize, ((self.M as f32) / (2 as f32)).ceil() as usize);
        let assigned_entries = self.pick_seeds(id);
        let mut groups = self.pick_next(id,&assigned_entries);
        let spelt_node_1 = groups.get(0).unwrap();
        let spelt_node_2 = groups.get(1).unwrap();
        let new_node = self.get_node(self.total_id+1);
    }

    fn split(& mut self, id: usize, mut assigned_1: Vec<&usize>, mut assigned_2: Vec<&usize>){
        //S1: pick first entry for each group, PickSeeds
        //m <= M/2 && m >= 2
        let m = max(2 as usize, ((M as f32) / (2 as f32)).ceil() as usize);
        let i: usize = assigned_1.len() + assigned_2.len();
        let first_one = self.pick_seeds(id).get(i).unwrap();
        assigned_1.push(first_one);
        let first_two = self.pick_seeds(id).get(i + 1).unwrap();
        assigned_2.push(first_two);
        //S2: check if done
        if assigned_1.len() + assigned_2.len() = self.total_id {}

        //If one group has so few entries
        //that all the rest must be assigned to it
        //in order for it to have the mini-mum number m, assign them and stop
        if self.total_id - assigned_1.len() == m {
            let id: usize = assigned_1.len() + assigned_2.len();
            for i in id..self.total_id - 1{
                let remaining = self.pick_seeds(id).get(i).unwrap();
                assigned_2.push(remaining);
            }
        }

        if self.total_id - assigned_2.len() == m {
            let id: usize = assigned_1.len() + assigned_2.len();
            for i in id..self.total_id - 1{
                let remaining = self.pick_seeds(id).get(i).unwrap();
                assigned_1.push(remaining);
            }
        }
        //S3: select entry to assign, PickNext, repeat S2


    }

    //TODO FERTIG
    fn area_two_rect(&mut self, rect1:&MBRect, rect2:&MBRect) -> f64 {
        let bl1 = &rect1.botton_left;
        let bl2 = &rect2.botton_left;
        let tr1 = &rect1.top_right;
        let tr2 = &rect2.top_right;

        let new_bl_x = min(bl1.x,bl2.x);
        let new_bl_y = min(bl1.y,bl2.y);
        let new_tr_x = max(tr1.x,tr2.x);
        let new_tr_y = max(tr1.y,tr2.y);

        let area = (new_tr_x - new_bl_x) * (new_tr_y - new_bl_y);
        area
    }

    //TODO FERTIG
    //Hilfsfkt fuer split
    //select two entries to be the first elements of the groups
    fn pick_seeds(&mut self, id:usize) -> Vec<usize>{
        let mut res:Vec<usize> = Vec::new();
        let spelt_node = self.get_node(id);
        let mut pick_group = spelt_node.children().unwrap();
        let mut largest_d = 0 as f64;
        let mut s1 = 0 as usize;
        let mut s2 = 0 as usize;
        for i in 0..pick_group.len()-2{
            for j in 1..pick_group.len()-1{
                let id_1 = pick_group[i];
                let id_2 = pick_group[j];
                let pick_rect_1 = self.get_rect(id_1);
                let pick_rect_2 = self.get_rect(id_2);
                let blank_area = self.area_two_rect(&pick_rect_1,&pick_rect_2) - self.rect_area_id(id_1) - self.rect_area_id(id_2);
                if blank_area > largest_d {
                    largest_d = blank_area;
                    s1 = id_1;
                    s2 = id_2;
                }
            }
        }
        res.push(s1);
        res.push(s2);
        res
    }

    //TODO FERTIG
    //Hilfsfkt fuer split
    //select remaining entries for classification in groups
    fn pick_next(&mut self, id:usize, assigned:&Vec<usize>) -> Vec<Vec<usize>>{
        let mut res: Vec<Vec<usize>> = Vec::new();

        let assigned_rect_1 = self.get_rect(assigned[0]);
        let assigned_rect_2 = self.get_rect(assigned[1]);

        let mut assigned_1:Vec<usize> = Vec::new();
        assigned_1.push(assigned[0]);
        let mut assigned_2:Vec<usize> = Vec::new();
        assigned_2.push(assigned[1]);

        let mut children = self.get_node(id).children().unwrap();

        let index_first_entry = pick_group.iter().position(|x| x == assigned_entries[0]).unwrap();
        let index_second_entry = pick_group.iter().position(|x| x == assigned_entries[1]).unwrap();

        children.remove(index_first_entry);
        children.remove(index_second_entry);

        let assigning = children;

        for i in 0..assigning.len()-1{
            let tmp_rect = self.get_rect(assigning[i]);
            let d1 = self.area_two_rect(&tmp_rect,&assigned_rect_1) - self.rect_area(&tmp_rect) - self.rect_area(&assigned_rect_1);
            let d2 = self.area_two_rect(&tmp_rect,&assigned_rect_2) - self.rect_area(&tmp_rect) - self.rect_area(&assigned_rect_2);
            if d1 <= d2 {
                assigned_1.push(assigning[i]);
            }
            else {
                assigned_2.push(assigning[i]);
            }
        }
        res.push(assigned_1);
        res.push(assigned_2);
        res
    }


}






use serde::{Serialize, Deserialize};
use bincode::{serialize, deserialize};
use core::borrow::Borrow;
use std::cmp::{min, max};
use std::intrinsics::{breakpoint, ceilf64};
use std::panic::resume_unwind;


#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Student{
    first_name: String,
    last_name: String,
    matr_nr: u32,
}
impl Student {
    pub fn new(fname: &str, lname: &str, matrnr: u32) -> Self {
        Student {
            first_name: fname.to_string(),
            last_name: lname.to_string(),
            matr_nr: matrnr,
        }
    }

    pub fn serialize(&self) -> Vec<u8> {
        bincode::serialize(&self).unwrap()
    }

    pub fn vec_to_block(vec:Vec<u8>) -> Block{
        Block::new(vec)
    }

    pub fn deserialize(input: &Vec<u8>) -> Student {
        bincode::deserialize(input).unwrap()
    }

    /*extern crate serde;
    extern crate serde_json;
    use serde_derive::*;
    #[derive(Debug, Serialize, Deserialize)]
    pub struct Student {
        vorname: String,
        nachname: String,
        matrnr: usize
    }

    impl Student{
        pub fn new(vor:&str,nach:&str,nr:&str) -> Student{
            let vorname = String::from(vor);
            let nachname = String::from(nach);
            let matrnr = String::from(nr).parse::<usize>().unwrap();
            Student{vorname,nachname,matrnr}
        }

        pub fn serialize(&mut self) -> Option<Block>{
            let student_json = serde_json::to_vec(self);
            match student_json {
                Ok(student) => {
                    Some(Block::new(student))
                }
                Err(error) => None
            }
        }

        pub fn serialize1(&mut self) -> Block{
            let mut vec = Vec::new();

            self.vorname.as_bytes();
            self.nachname.as_bytes().to_vec();
            vec.push(self.matrnr as u8);

            let block = Block::new(vec);
            block
        }

        pub fn deserialize(block: Block) -> Option<Student>{
            let bytes = block.contents;
            let s = String::from_utf8(bytes).expect("Found invalid UTF-8");
            //println!("{}", s);
            let student = serde_json::from_str(&s);
            match student{
                Ok(student) => {
                    Some(student)
                }
                Err(error) => None
            }
        }*/
}


impl Block{
    pub fn new(contents:Vec<u8>) -> Block{
        Block{contents}
    }

}


impl BFA {
    pub fn new(block_size:usize, path:&str) -> BFA{

        let filepath = format!("{}",path);
        let updatepath = format!("{}updated",path);
        let metadatenpath = format!("{}metadaten",path);

        let mut file = OpenOptions::new()
            .write(true)
            .read(true)
            .open(path);

        match file {
            //zwei faelle
            //1. File update_file existiert schon
            //bildet vector update_file aus dem File update_file

            Ok(mut file) => {
                Some(&file);

                let mut update_file: Vec<bool> = vec![true; block_size];
                let update = File::open(& updatepath);
                match update {
                    Ok(mut updated) => {
                        let mut vec: Vec<u8> = vec![0; file.metadata().unwrap().len() as usize];
                        updated.seek(SeekFrom::Start(0));
                        updated.read(&mut vec);

                        for i in 0..vec.len() {
                            if vec[i] == 0 {
                                update_file[i] = false;
                            }
                        }
                    }
                    Err(e) => {
                        println!("not exists")
                    }
                }

                let reserved_file: Vec<bool> = Vec::new();
                let mut metadaten = HashMap::new();
                metadaten.insert("path".to_string(),path.to_string());
                metadaten.insert("updated".to_string(),updatepath.to_string());
                metadaten.insert("metadaten".to_string(),metadatenpath.to_string());

                let reserve_count = update_file.len();

                BFA { block_size, file, metadaten, update_file, reserved_file, reserve_count }
            }
            //1. File update_file extiert schon
            //bildet vector update_file ganz neu
            Err(error) => {
                let new = OpenOptions::new()
                    .read(true)
                    .write(true)
                    .create(true)
                    .open(path).expect("error");

                let update_file: Vec<bool> = vec![true; block_size];
                let reserved_file: Vec<bool> = Vec::new();
                let mut metadaten = HashMap::new();
                metadaten.insert("path".to_string(),path.to_string());
                metadaten.insert("updated".to_string(),updatepath.to_string());
                metadaten.insert("metadaten".to_string(),metadatenpath.to_string());
                let reserve_count = 0 as usize;

                BFA { block_size, file: new, metadaten, update_file, reserved_file, reserve_count }
            }
        }


        /*let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path).expect("error");

        let update_file: Vec<bool> = vec![true;block_size];
        let reserved_file: Vec<bool> = Vec::new();
        let mut metadaten = HashMap::new();
        let reserve_count = 0 as usize;

        BFA{block_size,file,metadaten,update_file,reserved_file,reserve_count}*/
    }

    pub fn get(&mut self, &id:usize) -> Block{
        let mut vec:Vec<u8> = vec![0;self.block_size];

        if self.update_file[id] {
            let start = (id * self.block_size) as u64;

            self.file.seek(SeekFrom::Start(start));
            self.file.read(&mut vec);
            self.file.seek(SeekFrom::Start(0));
        }
        else { println!("update data not found") }

        let block = Block::new(vec);
        block
    }

    pub fn update(&mut self,id:usize, block: Block){
        //nach update, ist reserved_file[id] falsch, wird nie wieder zu true gesetzt
        //wenn man die noch benutzen will, guckt man dann im update_file nach
        if self.reserved_file[id] || self.update_file[id]{
            let start = (&id * self.block_size) as u64;
            self.file.seek(SeekFrom::Start(start));
            self.file.write(&block.contents);
            self.file.seek(SeekFrom::Start(0));
            self.reserved_file[&id] = false;
            self.update_file[&id] =true;
        }
    }

    pub fn insert(&mut self, block:Block) -> u64{
        let id = self.reserve();
        self.update(id,block);
        id as u64
    }

    pub fn contains(&mut self, id:usize) -> bool{
        let mut bool = false;
        if id > self.update_file.len() {
            println!("id: {} too large", id);
        }
        else{
            bool = self.update_file[id]
        }
        bool
    }

    pub fn remove(&mut self, id:usize) {
        if id > self.update_file.len() {
            println!("id: {} too large", id);
        }
        else {
            self.update_file[id] = false;
        }
    }

    pub fn reserve(&mut self) -> usize{
        let count = self.reserve_count;
        self.reserved_file.insert(count,true);
        self.reserve_count += 1;
        count
    }



    pub fn close(&mut self){
        self.reserved_file =  Vec::new();
        let mut updated_file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open("updated file")
            .expect("error");

        for i in 0 .. self.update_file.len() {
            if self.update_file[i]{
                write!(updated_file,"1");
            } else {
                write!(updated_file,"0");
            }
        }
        //self.update_file = vec![true;self.block_size];

    }

    pub fn get_metadaten(&mut self) {
        /*let mut daten = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open("metadaten.txt").expect("error");

        let metadata = self.file.metadata().expect("error");
        self.metadaten.insert(String::from("Length"), format!(":{}", metadata.len()));
        self.metadaten.insert("Is dir".to_string(),format!(":{}",metadata.is_dir()));

        for key in self.metadaten.keys(){
            let mut value = self.metadaten.get(key).expect("error");
           // print!("{}", value);
            daten.write(key.as_bytes());
            daten.write(value.as_bytes());
            daten.write_all(b"\n");
        }*/

    }

    pub fn get_root(& mut self) -> usize{
        let mut root = self.metadaten.get("path").expect("no root");
        let root = root.parse::<usize>().expect("invalid root");
        return root;
    }
}

#[cfg(test)]
mod test {
    use std::fs::File;
    use std::io::Read;
    use std::fmt::Error;
    use super::*;



    #[test]
    fn test_bfa_get_ok() -> Result<(),Error>{
        let block_size = 5 as usize;
        let mut file = File::create("Hello.txt").expect("error");
        file.write_all(b"HelloWorld").expect("error");
        let mut bfa_1 = BFA::new(block_size,"Hello.txt");
        let mut block = bfa_1.get(0);
        assert_eq!(block.contents, [72, 101, 108, 108, 111]);
        Ok(())
    }

    #[test]
    fn test_reserve_ok() -> Result<(),Error>{
        let block_size = 5 as usize;
        let mut file = File::create("Hello.txt").expect("error");
        let mut bfa_1 = BFA::new(block_size,"Hello.txt");
        file.write_all(b"HelloWorld").expect("error");
        bfa_1.reserve();
        bfa_1.reserve();
        assert_eq!(bfa_1.reserved_file,[true,true]);
        Ok(())
    }

    #[test]
    fn test_update_ok() -> Result<(),Error>{
        let block_size = 5;
        let mut file = File::create("Hello.txt").expect("error");
        let mut bfa_1 = BFA::new(block_size,"Hello.txt");
        bfa_1.file.write_all(b"HelloWorld").expect("error");
        let block_1 =bfa_1.get(0);
        bfa_1.reserve();
        bfa_1.reserve();
        bfa_1.update(1,block_1);

        let mut b = String::new();
        bfa_1.file.read_to_string(& mut b);
        assert_eq!(b, "HelloHello".to_string());
        Ok(())
    }

    #[test]
    fn test_get_metadaten_ok() -> Result<(),Error>{
        let mut file = File::create("Hello.txt").expect("error");
        let mut bfa_1 = BFA::new(5, "Hello.txt");
        file.write_all(b"HelloWorld").expect("error");
        bfa_1.get_metadaten();

        Ok(())
    }

    /*#[test]
    fn test_student_block_ok() -> Result<(),Error>{
        let mut student_1 = Student::new("Ling","Feng",2719983);
        let mut student_2 = Student::new("Yanping","Long",2767970);

        let student_1_block = student_1.serialize().unwrap();
        let bytes_1 = student_1_block.contents;
        let s_1 = String::from_utf8(bytes_1).expect("Found invalid UTF-8");
        assert_eq!(s_1,"{\"vorname\":\"Ling\",\"nachname\":\"Feng\",\"matrnr\":2719983}");

        let student_2_block = student_2.serialize().unwrap();
        let bytes_2 = student_2_block.contents;
        let s_2 = String::from_utf8(bytes_2).expect("Found invalid UTF-8");
        assert_ne!(s_2,"{\"vorname\":\"Ling\",\"nachname\":\"Feng\",\"matrnr\":2719983}");

        let student_3_block = student_1.serialize().unwrap();
        let student_3 = Student::deserialize(student_3_block).unwrap();
        assert_eq!(student_1.vorname,student_3.vorname);
        assert_eq!(student_1.nachname,student_3.nachname);
        assert_eq!(student_1.matrnr,student_3.matrnr);

        Ok(())
    }*/

    #[test]
    fn test_student_ok(){
        let s1 = Student::new("ling", "feng", 2719983);
        let s2 = Student::new("yanping", "long", 2767970);
        let serialized1:Vec<u8> = s1.serialize();
        let serialized2:Vec<u8> = s2.serialize();
        let deserialized1:Student = Student::deserialize(&serialized1);
        let deserialized2:Student = Student::deserialize(&serialized2);
        assert_eq!(s1, deserialized1);
        assert_eq!(s2, deserialized2);

    }



}



