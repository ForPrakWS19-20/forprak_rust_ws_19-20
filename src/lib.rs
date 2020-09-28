#![allow(unused)]
use std::fs::{File, OpenOptions, Metadata, metadata};
use std::io::{SeekFrom, Seek, Read, Write};
use std::{fs, mem};
use std::iter::Map;
use std::collections::HashMap;
use std::time::SystemTime;
use std::error::Error;


// BFA, Block file access, bietet die Moeglichkeit, Block zu get und put
// Ein Block hat eine einzige ID, richtet nach einem Bereich von xxByte nach xxByte in File
// Gebe bestimmte ID, kriege den Block, kriege den Teil v  cxdsaq   c   on File

pub struct BFA{
    pub block_size: usize,
    pub file: File,
    //metadata sollte auf Typ Map sein
    pub metadaten: HashMap<String,String>,
    //1 for true, 0 for false
    pub update_file: Vec<bool>,
    pub reserved_file: HashMap<usize,bool>,
    pub reserve_count:usize
}

pub struct Block{
    pub contents: Vec<u8>,
}

pub struct RTree{
    pub root_id:usize,
    pub bfa: BFA,
    dimension:usize,
    total_id: usize,
    M: usize
}


#[derive(Debug, Deserialize, Serialize,Clone)]
pub enum Node{
    //mittels von der id aus inneren Knoten das Blatt(als Block) ausholen
    //dann zum Blatt serialisieren
    Leaf{
        content: Vec<Point>
        //id
    },
    InnerNode{
        content: Vec<InnerElement>
    },
}

#[derive(Debug, Deserialize, Serialize,Clone,Copy)]
pub struct InnerElement {
    pub mbr:MBRect,
    //ID, mit welcher Blöcke vom BFA geholt werden können
    pub children:usize
}

#[derive(Debug, Deserialize, Serialize,Clone,Copy)]
pub struct Point{
    //Pos 0: x, Pos 1: y
    //coor:Vec<f64>
    x:f64,
    y:f64
}

#[derive(Debug, Deserialize, Serialize,Clone,Copy)]
pub struct MBRect{
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

    pub fn get_leaf_content(&mut self) -> Option<&mut Vec<Point>> {
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

    pub fn set_leaf_content(&mut self, new_content: Vec<Point>){
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

}

impl InnerElement{
    pub fn new(mbr: MBRect,children: usize) -> Self {
        InnerElement{ mbr, children }
    }

    pub fn set_mbr(&mut self, mbr:MBRect) {
        self.mbr = mbr;
    }

    pub fn equal(&self, other:&InnerElement) -> bool {
        return self.children == other.children && self.mbr.equal(&other.mbr)
    }
}

/*
impl LeafElement{
    pub fn new(daten: Vec<Point>,mbr: MBRect) -> Self {
        LeafElement{daten,mbr}
    }
}

 */

impl Point{
    pub fn new(x:f64, y:f64) -> Self{
        Point {x,y}
    }

    pub fn equal(&self,other:&Point) -> bool {
        return self.x == other.x && self.y == other.y
    }
}

impl MBRect{
    pub fn new(bl:Point, tp:Point) -> Self{
        MBRect{botton_left: bl, top_right: tp }
    }

    pub fn equal (&self, other:&MBRect) -> bool {
        return self.top_right.equal(&other.top_right) &&
            self.botton_left.equal(&other.botton_left)
    }

    fn mbr_of_rects(&self, another:&MBRect) -> MBRect{
        let minx1 = self.botton_left.x;
        let miny1 = self.botton_left.y;
        let maxx1 = self.top_right.x;
        let maxy1 = self.top_right.y;
        let minx2 = another.botton_left.x;
        let miny2 = another.botton_left.y;
        let maxx2 = another.top_right.x;
        let maxy2 = another.top_right.y;
        let minx = minx1.min(minx2);
        let miny = miny1.min(miny2);
        let maxx = maxx1.max(maxx2);
        let maxy = maxy1.max(maxy2);
        let min= Point::new(minx,miny);
        let max= Point::new(maxx,maxy);
        let rect: MBRect = MBRect::new(min,max);
        rect
    }

    fn mbr_of_point_and_rect(&self, point:&Point) -> MBRect{
        let minx1 = self.botton_left.x;
        let miny1 = self.botton_left.y;
        let maxx1 = self.top_right.x;
        let maxy1 = self.top_right.y;
        let x2 = point.x;
        let y2 = point.y;
        let minx = minx1.min(x2);
        let miny = miny1.min(y2);
        let maxx = maxx1.max(x2);
        let maxy = maxy1.max(y2);
        let min= Point::new(minx,miny);
        let max= Point::new(maxx,maxy);
        let mbr = MBRect::new(min,max);
        return mbr;
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
        let minx = minx1.max(minx2);
        let miny = miny1.max(miny2);
        let maxx = maxx1.min(maxx2);
        let maxy = maxy1.min(maxy2);
        return (minx < maxx) && (miny < maxy);
    }

    pub fn rect_area (&self) -> f64 {
        let minx = self.botton_left.x;
        let miny = self.botton_left.y;
        let maxx = self.top_right.x;
        let maxy = self.top_right.y;
        let area = (maxy - miny) * (maxx - minx);
        return area;
    }

    fn point_in_rect(&self, daten:&Point) -> bool {
        return daten.x <= self.top_right.x && daten.x >= self.botton_left.x &&
            daten.y <= self.top_right.y && daten.y >= self.botton_left.y;
    }

    fn add_area(&self, daten:&Point) -> f64 {
        let mut area = 0 as f64;
        if self.point_in_rect(daten) {}
        else {
            let minx = self.botton_left.x.min(daten.x);
            let miny = self.botton_left.y.min(daten.y);
            let maxx = self.top_right.x.max(daten.x);
            let maxy = self.top_right.y.max(daten.y);
            let new_rect_area = (maxy - miny) * (maxx - minx);
            area = new_rect_area - self.rect_area();
        }
        return area;
    }
}


impl RTree{
    pub fn new(M: usize, path:&str, block_size:usize) -> Self{
        let bfa = BFA::new(block_size, path);
        let dimension: usize = 2;

        RTree{
            root_id: 0,
            bfa,
            dimension,
            total_id: 0,
            M
        }
    }

    pub fn node_is_leaf(&mut self, node: &Node) -> bool{
        match node {
            Node::Leaf {content} => true,
            Node::InnerNode {content} => false,
        }
    }

//////////////////////////////////////////////////////////////////////////////////////////
    //Basis Funktion
pub fn get_node(&mut self, id: usize) -> Node {
        let mut block = self.bfa.get(id).unwrap();
        let node = Node::from_block(& mut block);
        return node;
    }

    pub fn mbr_of_points(&mut self, points:&mut Vec<Point>, id:usize) -> MBRect{
        let mut minx = points.first().unwrap().x;
        let mut miny = points.first().unwrap().y;
        let mut maxx = points.first().unwrap().x;
        let mut maxy = points.first().unwrap().y;
        for i in points {
            minx = minx.min(i.x);
            miny = minx.min(i.y);
            maxx = maxx.max(i.x);
            maxy = maxx.max(i.y);
        }
        let bl = Point::new(minx,miny);
        let tr = Point::new(maxx,maxy);
        let rect = MBRect::new(bl,tr);
        return rect;
    }

    pub fn get_leaf_points(&mut self, id: usize) -> Option<Vec<Point>> {
        let node = self.get_node(id);
        match node {
            Node::Leaf {content} => {
                let mut res: Vec<Point> = Vec::new();
                for i in content {
                    res.push(i);
                }
                Some(res)
            }
            Node::InnerNode {content} => {
                None
            }
        }
    }

    fn get_innernode_rect(&mut self, id: usize) -> Option<Vec<MBRect>>{
        let node = self.get_node(id);
        match node {
            Node::Leaf {content} => {
                None
            }
            Node::InnerNode {content} => {
                let mut res: Vec<MBRect> = Vec::new();
                for i in content {
                    res.push(i.mbr);
                }
                Some(res)
            }
        }
    }

    fn traverse(&mut self, id:usize,mut res:Vec<Point>) -> Option<Vec<Point>>{
        let tmp = self.get_node(id);
        match tmp {
            Node::Leaf {content } => {
                let mut contents = content.clone();
                contents.reverse();
                while contents.len()!=0 {
                    res.push(contents.pop().unwrap());
                }
            }
            Node::InnerNode {content} => {
                for i in 0..content.len() {
                    let child = content.get(i).unwrap().children;
                    let result = res.clone();
                    let mut vec = self.traverse(child,result).unwrap();
                    res.append(&mut vec);
                }
            }
        }
        return Some(res);
    }

/////////////////////////////////////////////////////////////////////////////////////////

    fn search_overlap_innernode(&mut self, rect: &MBRect, tmp: usize, mut overlapped: Vec<usize>) -> Option<Vec<usize>>{
        let mut tmp_node = self.get_node(tmp);
        match tmp_node {
            Node::Leaf { content } => { None }
            Node::InnerNode { content } => {
                for i in content {
                    let mut status = false;
                    if rect.overlap(&i.mbr) {
                        let children_node = self.get_node(i.children);
                        match children_node {
                            Node::Leaf { content } => { overlapped.push(i.children) }
                            Node::InnerNode { content } => {
                                overlapped.append(&mut self.search_overlap_innernode(rect, i.children, overlapped.clone()).unwrap());
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
                for i in content {
                    if rect.point_in_rect(&i) {
                        res.push(i);
                    }
                }
                Some(res)
            }
        }

    }

    pub fn search(&mut self, rect: &MBRect) -> Option<Vec<Point>>{
        //生产根Block，Node和Rect
        //Erstelle Block, Node von der Wurzel
        let root_id = self.root_id;
        let root_node = self.get_node(root_id);
        let mut res:Option<Vec<Point>> = None;
        match root_node {
            Node::Leaf {content} => {
                res = self.search_overlap_leafnode(rect,root_id);
            }
            //Node ist InnerNode
            Node::InnerNode {content} => {
                let overlapped = self.search_overlap_innernode(rect,root_id,Vec::new()).unwrap();
                let mut res_point: Vec<Point> = Vec::new();
                for i in overlapped {
                    let point = self.search_overlap_leafnode(rect,i).unwrap();
                    for j in point {
                        res_point.push(j);
                    }
                }
                res = Some(res_point);
            }
        }
        return res;
    }


    ////////////////////////////////////////////////////////////////////////////////
    pub fn insert(&mut self, insert_daten: Point) {
        if self.bfa.reserve_count == 0 {
            let mut node = Node::Leaf {content:Vec::new()};
            let block = Block::to_block(&mut node);
            self.bfa.insert(block);
        }
        self.r_tree_insert(insert_daten);
    }

    //insert a new node into RTree
    fn r_tree_insert(&mut self, insert_daten:Point){
        let mut parent: Vec<usize> = Vec::new();
        let leaf_id = self.choose_leaf(self.root_id, &insert_daten, &mut parent).0;
        let mut id_ancestry = self.choose_leaf(self.root_id, &insert_daten, &mut parent).1;

        self.add_point_in_leaf(leaf_id,insert_daten);

        let mut leaf_node = self.get_node(leaf_id);
        let mut leaf_elem_num = leaf_node.get_leaf_content().unwrap().len();

        if leaf_elem_num <= self.M {
            let block = Block::to_block(&mut leaf_node);
            self.bfa.update(leaf_id,block);
            self.adjust_tree(leaf_id, &mut id_ancestry, false, 0);
        }
        else {
            let spelt_id = self.split(leaf_id);
            self.adjust_tree(leaf_id, &mut id_ancestry, true, spelt_id)
        }
    }

    pub fn add_point_in_leaf(&mut self, leaf_id:usize, daten:Point) {
        let mut leaf_node = self.get_node(leaf_id);
        leaf_node.get_leaf_content().unwrap().push(daten);
        let leaf_block = Block::to_block(&mut leaf_node);
        self.bfa.update(leaf_id,leaf_block);
    }

    /*
    //Hilfsfunktion
    fn add_rect_in_leaf(&mut self,leaf_id:usize,rect:MBRect) {
        let mut leaf_node = self.get_node(leaf_id);
        let new_leaf = LeafElement{ daten: vec![], mbr: rect };
        leaf_node.get_leaf_content().unwrap().push(new_leaf);
    }
    */



    /*
    //Hilfsfunktion fuer choose_leaf und choose_leaves
    fn add_area(&mut self, small_rect: &MBRect, big_rect: &MBRect) -> f64{
        let new = small_rect.mbr_of_rects(big_rect);
        let add_area = new.rect_area() - big_rect.rect_area();
        add_area
    }
     */


    //Hilfsfunktion fuer insert
    //Find position for new record
    pub fn choose_leaf(&mut self, tmp: usize, insert_daten:&Point, parent: &mut Vec<usize>) -> (usize, Vec<usize>) {
        let mut tmp_node = self.get_node(tmp);
        if !self.node_is_leaf(&tmp_node) {
            let mut min_add_area = tmp_node.get_innernode_content().unwrap().get(0).unwrap().mbr.add_area(insert_daten);
            let mut min_node_id = tmp_node.get_innernode_content().unwrap().get(0).unwrap().children;
            let mut min_node_area = tmp_node.get_innernode_content().unwrap().get(0).unwrap().mbr.rect_area();
            for i in tmp_node.get_innernode_content().unwrap() {
                if i.mbr.add_area(insert_daten) < min_add_area {
                    min_add_area = i.mbr.add_area(insert_daten);
                    min_node_id = i.children;
                    min_node_area = i.mbr.rect_area();
                }
                else if i.mbr.add_area(insert_daten) == min_node_area {
                    if i.mbr.rect_area() < min_node_area {
                        min_node_id = i.children;
                        min_node_area = i.mbr.rect_area();
                    }
                 }
            }
            parent.push(tmp);
            self.choose_leaf(min_node_id,insert_daten,parent)
        }
        else {
            return (tmp,parent.to_vec());
        }
    }

    //TODO
    //Hilfsfunktion fuer adjust
    pub fn get_node_mbr(&mut self, node_id:usize) -> MBRect {
        let mut node = self.get_node(node_id);
        match node {
            Node::Leaf {content} => {
                let mut points = content.clone();
                if points.len() ==1 {
                    let mbr = MBRect::new(*points.get(0).unwrap(),*points.get(0).unwrap());
                    return mbr
                }else{
                    let p1= points.pop().unwrap();
                    let p2 =points.pop().unwrap();
                    let mut vp= vec![p1,p2];
                    let mut mbr = self.mbr_of_points(&mut vp,node_id);
                    for i in points{
                        mbr = mbr.mbr_of_point_and_rect(&i);
                    }
                    return mbr;
                }
            }
            Node::InnerNode {content} => {
                let mbrs = self.get_innernode_rect(node_id).unwrap();
                let mut mbrs_clone = mbrs.clone();
                if mbrs_clone.len() == 1 {
                    return *mbrs_clone.get(0).unwrap();
                }else{
                    let mut mbr = mbrs_clone.pop().unwrap();
                    while mbrs_clone.len() != 1 {
                        mbr = mbr.mbr_of_rects(&mbrs_clone.pop().unwrap());
                    }
                    return mbr;
                }
            }
        }
    }

    //acsend from a leaf node with id to the root
    //adjusting covering rectangles
    fn adjust_tree(& mut self, id: usize, id_ancestry: &mut Vec<usize>, soll_spelt:bool, split: usize){
        let mut id_node = self.get_node(id);
        //let mut parent = id_ancestry.pop().unwrap();
        //let mut parent_node = self.get_node(parent);
        //check if done, stop
        if id == self.root_id{
            if !soll_spelt {}
            else {
                //split root
                let EN = InnerElement::new(self.get_node_mbr(id),id);
                let ENN = InnerElement::new(self.get_node_mbr(split),split);
                let mut new_root_node = Node::InnerNode { content: vec![EN,ENN] };
                let new_root_id = self.bfa.reserve();
                let mut new_root_block = Block::to_block(&mut new_root_node);
                self.bfa.update(new_root_id,new_root_block);
                self.root_id = new_root_id;
            }
        }
        else {
            let mut parent = id_ancestry.pop().unwrap();
            let mut parent_node = self.get_node(parent);
            if !soll_spelt {
                for i in parent_node.get_innernode_content().unwrap() {
                    if i.children == id {
                        i.set_mbr(self.get_node_mbr(id));
                    }
                }
                let mut parent_block = Block::to_block(&mut parent_node);
                self.bfa.update(parent,parent_block);
                self.adjust_tree(parent,id_ancestry,false,0)
            }
            else {
                for i in parent_node.get_innernode_content().unwrap() {
                    if i.children == id {
                        i.set_mbr(self.get_node_mbr(id));
                    }
                }
                let new = InnerElement::new(self.get_node_mbr(split),split);
                parent_node.get_innernode_content().unwrap().push(new);
                let mut parent_block = Block::to_block(&mut parent_node);
                self.bfa.update(parent,parent_block);
                if parent_node.get_innernode_content().unwrap().len() <= self.M {
                    self.adjust_tree(parent,id_ancestry,false,0)
                }
                else {
                    let spelt_id = self.split(parent);
                    self.adjust_tree(parent,id_ancestry,true,spelt_id);
                }
            }
        }

    }

    pub fn split(&mut self, id: usize) -> usize{
        let mut groups = self.pick_next(id);
        let group_1_index = groups.get(0).unwrap();
        let group_2_index = groups.get(1).unwrap();
        let mut node = self.get_node(id);
        let mut node_vec: Vec<Node> = vec![];

        match node {
            Node::InnerNode {mut content} => {
                let mut content_clone = content.clone();
                let mut group_1_node = Node::InnerNode { content: vec![] };
                let mut group_2_node = Node::InnerNode { content: vec![] };
                for i in group_1_index {
                    let elem = content.get(*i).unwrap();
                    //while !content_clone.get(content_clone.len()-1).unwrap().equal(elem) {
                    //    content_clone.pop();
                    //}
                    group_1_node.get_innernode_content().unwrap().push(content_clone.pop().unwrap());
                }
                for i in group_2_index {
                    let elem = content.get(*i).unwrap();
                    //while !content_clone.get(content_clone.len()-1).unwrap().equal(elem) {
                    //    content_clone.pop();
                    //}
                    group_2_node.get_innernode_content().unwrap().push(content_clone.pop().unwrap());
                }
                node_vec = vec![group_1_node,group_2_node];
            }
            Node::Leaf {content} => {
                let mut content_clone = content.clone();
                let mut group_1_node = Node::Leaf { content: vec![] };
                let mut group_2_node = Node::Leaf { content: vec![] };
                for i in group_1_index {
                    let elem = *content.get(*i).unwrap();
                    //while !content_clone.get(content_clone.len()-1).unwrap().equal(elem) {
                    //    content_clone.pop();
                    //}
                    group_1_node.get_leaf_content().unwrap().push(elem);
                }
                for i in group_2_index {
                    let elem = *content.get(*i).unwrap();
                    //while !content_clone.get(content_clone.len()-1).unwrap().equal(elem) {
                    //    content_clone.pop();
                    //}
                    group_2_node.get_leaf_content().unwrap().push(elem);
                }
                node_vec = vec![group_1_node,group_2_node];
            }
        }
        let node_2_id = self.bfa.reserve();
        let block_2 = Block::to_block(&mut node_vec.get(1).unwrap());
        self.bfa.update(node_2_id,block_2);
        let block_1 = Block::to_block(&mut node_vec.get(0).unwrap());
        self.bfa.update(id,block_1);
        //LeafNode kann als InnerElement als parentNode (InnerNode) sein
        //InnerNode kann auch als InnerElement sein
        return node_2_id;
    }

    fn area_two_rect(&mut self, rect1:&MBRect, rect2:&MBRect) -> f64 {
        let union_rect = rect1.mbr_of_rects(rect2);
        let area = union_rect.rect_area();
        area
    }


    //Hilfsfkt fuer split
    //select two entries to be the first elements of the groups
    pub fn pick_seeds(&mut self, id:usize) -> Vec<usize>{
        let mut res:Vec<usize> = Vec::new();
        let spelt_node = self.get_node(id);
        let mut largest_d = 0 as f64;
        let mut s1: usize = 0;
        let mut s2: usize = 0;

        match spelt_node {
            Node::InnerNode {content} => {
                for i in 0..content.len()-1  {
                    for j in 1..content.len() {
                        let pick_rect_1 = &content.get(i).unwrap().mbr;
                        let pick_rect_2 = &content.get(j).unwrap().mbr;
                        let blank_area = self.area_two_rect(&pick_rect_1, &pick_rect_2) - pick_rect_1.rect_area() - pick_rect_2.rect_area();
                        if blank_area > largest_d {
                            largest_d = blank_area;
                            s1 = i;
                            s2 = j;
                        }
                    }
                }
            }
            Node::Leaf {content} => {
                for i in 0..content.len()-1  {
                    for j in 1..content.len()  {
                        let pick_rect_1 = *content.get(i).unwrap();
                        let pick_rect_2 = *content.get(j).unwrap();
                        let mut points = vec![pick_rect_1, pick_rect_2];
                        let blank_area = self.mbr_of_points(&mut points, id).rect_area();
                        if blank_area > largest_d {
                            largest_d = blank_area;
                            s1 = i;
                            s2 = j;
                        }
                    }
                }
            }
        }
        res.push(s1);
        res.push(s2);
        res
    }


    //Hilfsfkt fuer split
    //select remaining entries for classification in groups
    pub fn pick_next(&mut self, id:usize) -> Vec<Vec<usize>>{
        let mut res: Vec<Vec<usize>> = Vec::new();
        let spelt_node = self.get_node(id);
        let seed1 = self.pick_seeds(id)[0];
        let seed2 = self.pick_seeds(id)[1];

        let mut assigned_1:Vec<usize> = Vec::new();
        let mut assigned_2:Vec<usize> = Vec::new();

        match spelt_node {
            Node::InnerNode { content } => {
                let assigned_rect_1 = &content.get(seed1).unwrap().mbr;
                let assigned_rect_2 = &content.get(seed2).unwrap().mbr;

                for i in 0..content.len() {
                    let tmp_rect = &content.get(i).unwrap().mbr;
                    //seed1 kommt vor, d1 = - seed1.mbr.area, d1 < d2 erfüllt
                    //seed1 wird in assigned 1 hinzufügt
                    let d1 = self.area_two_rect(&tmp_rect, &assigned_rect_1) - tmp_rect.rect_area() - assigned_rect_1.rect_area();
                    let d2 = self.area_two_rect(&tmp_rect, &assigned_rect_2) - tmp_rect.rect_area() - assigned_rect_2.rect_area();
                    if d1 <= d2 {
                        assigned_1.push(i);
                    } else {
                        assigned_2.push(i);
                    }
                }
            }
            Node::Leaf { content } => {
                let assigned_point_1 = *content.get(seed1).unwrap();
                let assigned_point_2 = *content.get(seed2).unwrap();

                for i in 0..content.len() {
                    let tmp_point = *content.get(i).unwrap();
                    let mut points1 = vec![tmp_point,assigned_point_1];
                    let mut points2 = vec![tmp_point,assigned_point_2];
                    let d1 = self.mbr_of_points(&mut points1,id).rect_area();
                    let d2 = self.mbr_of_points(&mut points2,id).rect_area();
                    if d1 <= d2 {
                        assigned_1.push(i);
                    } else {
                        assigned_2.push(i);
                    }
                }
            }
        }

        res.push(assigned_1);
        res.push(assigned_2);
        res
    }


    pub fn insert_into_node(&mut self, elem: InnerElement, id:usize, data: Vec<Point>) -> bool {
        let tmp = self.get_node(id);
        match tmp {
            Node::InnerNode {mut content} => {
                if content.len() < self.M{
                    content.push(elem);
                    let mut new_tmp = Node::InnerNode {content};
                    let block = Block::to_block(&mut new_tmp);
                    self.bfa.update(id,block);
                    return true;
                }
                else { return false; }
            }
            Node::Leaf {content} => {
                return false;
            }
        }
    }
}






use serde::{Serialize, Deserialize};
use bincode::{serialize, deserialize};
use core::borrow::Borrow;
use std::cmp::{min, max};
//use std::intrinsics::{breakpoint, ceilf64};
use std::panic::resume_unwind;
use crate::Node::{InnerNode, Leaf};
use std::ops::BitAnd;
use std::slice::SliceIndex;
use std::hash::Hash;


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

    pub fn to_block<T> (object: &mut T) -> Self where T:serde::Serialize {
        let obj = bincode::serialize(object).unwrap();
        let block = Block::new(obj);
        block
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
            .open(&path);

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

                let reserved_file = HashMap::new();
                let mut metadaten = HashMap::new();
                metadaten.insert("path".to_string(),"0".to_string());
                metadaten.insert("updated".to_string(),updatepath);
                metadaten.insert("metadaten".to_string(),metadatenpath);

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
                let reserved_file = HashMap::new();
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

    pub fn get(&mut self, id:usize) -> Option<Block>{
        let mut vec:Vec<u8> = vec![0;self.block_size];

        if self.update_file[id] {
            let start = (id * self.block_size) as u64;

            self.file.seek(SeekFrom::Start(start)).expect("error bfa get");
            self.file.read(&mut vec).expect("error bfa get");
            self.file.seek(SeekFrom::Start(0)).expect("error bfa get");
            let block = Block::new(vec);
            Some(block)
        }
        else { None }
    }

    pub fn update(&mut self, id: usize, mut block: Block) -> Result<(), Box<dyn Error>> {
        if block.contents.len() > self.block_size {
            return Err("Block is too large".into());
        }
        else {
            let res = self.reserved_file.get(&id);
            //fill entire block, important for the last block
            if block.contents.len() < self.block_size {
                for _i in block.contents.len()..self.block_size {
                    block.contents.push(0);
                }
            }

            match res {
                Some(bool) => {
                    if bool == &true {
                        self.file.seek(SeekFrom::Start((id * self.block_size) as u64)).expect("error bfa update");
                        self.file.write(&block.contents)?;
                        self.update_file.insert(id,true);
                        self.reserved_file.remove(&id);
                    }
                    else {
                        return Err("id not reserved".into());
                    }
                }
                None => {
                    if self.update_file[id] {
                        self.file.seek(SeekFrom::Start((id * self.block_size) as u64)).expect("error bfa update");
                        self.file.write(&block.contents)?;
                        self.reserved_file.remove(&id);
                    }
                    else {
                        return Err("id not reserved".into());
                    }
                }
            }
            return Ok(())
        }
    }

    pub fn insert(&mut self, block:Block) -> u64{
        let id = self.reserve();
        self.update(id,block).expect("error bfa insert");
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
        self.reserved_file =  HashMap::new();
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
        let mut root_str = self.metadaten.get("path").expect("no root");
        let root = root_str.parse::<usize>().expect("invalid root");
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
    pub fn test_insert_without_split(){
        let mut rtree = RTree::new(4,"test_insert_without_split()",1000);
        let point1 = Point::new(1.0,1.0);
        rtree.insert(point1);
        let point2 = Point::new(2.0,2.0);
        rtree.insert(point2);
        let point3 = Point::new(3.0,3.0);
        rtree.insert(point3);
        let res:Vec<Point> = Vec::new();
        let v = vec![point1,point2,point3];
        let elem = rtree.traverse(0,res).unwrap();
        assert_eq!(rtree.root_id,0);
        assert_eq!(elem.first().unwrap().x,v.first().unwrap().x);
        assert_eq!(elem.get(1).unwrap().x,v.get(1).unwrap().x);
        assert_eq!(elem.last().unwrap().x,v.last().unwrap().x);
    }

    #[test]
    pub fn test_insert_with_split(){
        let mut rtree = RTree::new(4,"test_insert_with_split()",1000);
        let point1 = Point::new(1.0,1.0);
        rtree.insert(point1);
        let point2 = Point::new(2.0,2.0);
        rtree.insert(point2);
        let point3 = Point::new(3.0,3.0);
        rtree.insert(point3);
        let point4 = Point::new(4.0,4.0);
        rtree.insert(point4);
        let point5 = Point::new(5.0,5.0);
        rtree.insert(point5);
        let res:Vec<Point> = Vec::new();
        let v1 = vec![point1,point2,point3];
        let v2 = vec![point4,point5];
        assert_eq!(rtree.root_id,2);
        for i in 0..v1.len() {
            assert_eq!(rtree.get_node(0).get_leaf_content().unwrap().get(i).unwrap().x,v1.get(i).unwrap().x);
            assert_eq!(rtree.get_node(0).get_leaf_content().unwrap().get(i).unwrap().y,v1.get(i).unwrap().y);
        }
        for i in 0..v2.len() {
            assert_eq!(rtree.get_node(1).get_leaf_content().unwrap().get(i).unwrap().x,v2.get(i).unwrap().x);
            assert_eq!(rtree.get_node(1).get_leaf_content().unwrap().get(i).unwrap().y,v2.get(i).unwrap().y);
        }
        assert_eq!(rtree.get_node(2).get_innernode_content().unwrap().get(0).unwrap().children,0);
        assert_eq!(rtree.get_node(2).get_innernode_content().unwrap().get(1).unwrap().children,1);
    }

    #[test]
    pub fn test_search_without_split() {
        let mut rtree = RTree::new(4,"test_search_without_split",1000);
        let point1 = Point::new(1.0,1.0);
        rtree.insert(point1);
        let point2 = Point::new(2.0,2.0);
        rtree.insert(point2);
        let point3 = Point::new(3.0,3.0);
        rtree.insert(point3);
        let rect = MBRect::new(Point::new(0.5,0.5),Point::new(2.5,2.5));
        let search = rtree.search(&rect).unwrap();
        for i in 0..search.len() {
            assert_eq!(search.get(i).unwrap().x,vec![point1,point2].get(i).unwrap().x);
            assert_eq!(search.get(i).unwrap().y,vec![point1,point2].get(i).unwrap().y);
        }
    }

    #[test]
    pub fn test_search_with_split() {
        let mut rtree = RTree::new(4,"test_search_with_split",1000);
        let point1 = Point::new(1.0,1.0);
        rtree.insert(point1);
        let point2 = Point::new(2.0,2.0);
        rtree.insert(point2);
        let point3 = Point::new(3.0,3.0);
        rtree.insert(point3);
        let point4 = Point::new(4.0,4.0);
        rtree.insert(point4);
        let point5 = Point::new(5.0,5.0);
        rtree.insert(point5);
        let rect = MBRect::new(Point::new(0.5,0.5),Point::new(2.5,2.5));
        let search = rtree.search(&rect).unwrap();
        for i in 0..search.len() {
            assert_eq!(search.get(i).unwrap().x,vec![point1,point2].get(i).unwrap().x);
            assert_eq!(search.get(i).unwrap().y,vec![point1,point2].get(i).unwrap().y);
        }
    }

    #[test]
    pub fn test_search_none() {
        let mut rtree = RTree::new(4,"test_search_none",1000);
        let point1 = Point::new(1.0,1.0);
        rtree.insert(point1);
        let point2 = Point::new(2.0,2.0);
        rtree.insert(point2);
        let point3 = Point::new(3.0,3.0);
        rtree.insert(point3);
        let rect = MBRect::new(Point::new(1.2,1.2),Point::new(1.5,1.5));
        let search = rtree.search(&rect).unwrap();
        assert_eq!(search.len(),0);
    }

    #[test]
    fn test_bfa_get_ok() -> Result<(),Error>{
        let block_size = 5 as usize;
        let mut file = File::create("Hello.txt").expect("error");
        file.write_all(b"HelloWorld").expect("error");
        let mut bfa_1 = BFA::new(block_size,"Hello.txt");
        let mut block = bfa_1.get(0).unwrap();
        assert_eq!(block.contents, [72, 101, 108, 108, 111]);
        Ok(())
    }

    #[test]
    fn test_reserve_ok() -> Result<(),Error>{
        let block_size = 5 as usize;
        let mut file = File::create("Hello.txt").expect("error");
        let mut bfa_1 = BFA::new(block_size,"Hello.txt");
        file.write_all(b"HelloWorld").expect("error");
        let a = bfa_1.reserve();
        let b = bfa_1.reserve();
        assert_eq!(*bfa_1.reserved_file.get(&a).unwrap(),true);
        assert_eq!(*bfa_1.reserved_file.get(&b).unwrap(),true);
        Ok(())
    }

    #[test]
    fn test_update_ok() -> Result<(),Error>{
        let block_size = 5;
        let mut file = File::create("Hello.txt").expect("error");
        let mut bfa_1 = BFA::new(block_size,"Hello.txt");
        bfa_1.file.write_all(b"HelloWorld").expect("error");
        let block_1 =bfa_1.get(0).unwrap();
        bfa_1.update(1,block_1);

        assert_eq!(bfa_1.get(1).unwrap().contents, bfa_1.get(0).unwrap().contents);
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



